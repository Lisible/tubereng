use std::iter::Peekable;
use std::str::FromStr;

use crate::{GraphicsError, Result};
use tree_sitter::{Node, Parser};

pub struct Variable {
    pub attributes: Attributes,
    pub address_space: Option<AddressSpace>,
    pub access_mode: Option<AccessMode>,
    pub identifier: String,
    pub kind: VariableKind,
}

pub struct Attributes {
    pub group: u32,
    pub binding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSpace {
    Uniform,
    Storage,
}

impl TryFrom<&str> for AddressSpace {
    type Error = GraphicsError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "uniform" => Ok(AddressSpace::Uniform),
            "storage" => Ok(AddressSpace::Storage),
            _ => Err(GraphicsError::WgslAddressSpaceParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

impl TryFrom<&str> for AccessMode {
    type Error = GraphicsError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "read" => Ok(AccessMode::Read),
            "write" => Ok(AccessMode::Write),
            "read_write" => Ok(AccessMode::ReadWrite),
            _ => Err(GraphicsError::WgslAccessModeParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    Texture,
    Sampler,
    Other,
}

impl From<&str> for VariableKind {
    fn from(value: &str) -> Self {
        if value.starts_with("texture_2d<") {
            VariableKind::Texture
        } else if value == "sampler" {
            VariableKind::Sampler
        } else {
            VariableKind::Other
        }
    }
}

/// Parses the global variable from a wgsl shader
///
/// # Errors
/// This will return an Err if the parsing fails
pub fn extract_global_variables_from_shader_source(source: &str) -> Result<Vec<Variable>> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_wgsl::language())
        .map_err(|_| GraphicsError::WgslTreeSitterLanguageError)?;
    let tree = parser
        .parse(source, None)
        .ok_or(GraphicsError::WgslTreeSitterParseError)?;
    let root_node = tree.root_node();
    let mut cursor = tree.walk();

    let mut variables = vec![];
    for node in root_node.named_children(&mut cursor) {
        if node.kind() != "global_variable_decl" {
            continue;
        }

        if node
            .children(&mut node.walk())
            .filter(|c| c.kind() == "attribute")
            .count()
            == 0
        {
            continue;
        }

        variables.push(parse_variable(node, source)?);
    }

    Ok(variables)
}

fn parse_variable(node: Node, source: &str) -> Result<Variable> {
    let attributes = parse_attributes(node, source)?;
    let (address_space, access_mode, identifier, r#type) = parse_declaration(node, source)?;
    Ok(Variable {
        attributes,
        address_space,
        access_mode,
        identifier,
        kind: r#type,
    })
}

fn parse_declaration(
    node: Node,
    source: &str,
) -> Result<(
    Option<AddressSpace>,
    Option<AccessMode>,
    String,
    VariableKind,
)> {
    let mut address_space = None;
    let mut access_mode = None;
    let mut identifier = None;
    let mut variable_type = None;

    for child in node.children(&mut node.walk()) {
        if child.kind() != "variable_decl" {
            continue;
        }

        let mut cursor = child.walk();
        let mut variable_decl_children_iterator = child.children(&mut cursor).peekable();
        eat(&mut variable_decl_children_iterator, "var")?;
        let qualifier_node =
            parse_optional_node(&mut variable_decl_children_iterator, "variable_qualifier");
        if let Some(qualifier_node) = qualifier_node {
            (address_space, access_mode) = parse_qualifier(qualifier_node, source)?;
        }

        let identifier_node =
            parse_node(&mut variable_decl_children_iterator, "variable_ident_decl")?;
        (identifier, variable_type) = parse_identifier_and_type(identifier_node, source)?;
    }

    Ok((
        address_space,
        access_mode,
        identifier.unwrap(),
        variable_type.unwrap(),
    ))
}

fn parse_identifier_and_type(
    identifier_node: Node,
    source: &str,
) -> Result<(Option<String>, Option<VariableKind>)> {
    let mut cursor = identifier_node.walk();
    let mut it = identifier_node.children(&mut cursor);
    let ident_node = parse_node(&mut it, "ident")?;
    let identifier = node_source_string(source, ident_node)?;
    eat(&mut it, "colon")?;
    let type_decl_node = parse_node(&mut it, "type_decl")?;
    let variable_type = parse_type_decl(type_decl_node, source)?;
    Ok((Some(identifier.into()), Some(variable_type)))
}

fn parse_type_decl(type_decl_node: Node, source: &str) -> Result<VariableKind> {
    let mut cursor = type_decl_node.walk();
    let mut it = type_decl_node.children(&mut cursor).peekable();
    let mut ident_node = parse_optional_node(&mut it, "ident");
    if ident_node.is_none() {
        ident_node = parse_optional_node(&mut it, "type_decl_without_ident");
    }

    let ident_node = ident_node.ok_or(GraphicsError::WgslMissingTypeDecl)?;
    let identifier = node_source_string(source, ident_node)?;
    let variable_type = VariableKind::from(identifier);
    Ok(variable_type)
}

fn node_source_string<'a>(source: &'a str, ident_node: Node) -> Result<&'a str> {
    std::str::from_utf8(&source.as_bytes()[ident_node.byte_range()])
        .map_err(|_| GraphicsError::WgslParseSourceStringError)
}

fn parse_qualifier(
    qualifier_node: Node,
    source: &str,
) -> Result<(Option<AddressSpace>, Option<AccessMode>)> {
    let mut cursor = qualifier_node.walk();
    let mut it = qualifier_node.children(&mut cursor);
    eat(&mut it, "less_than")?;
    let address_space_node = parse_node(&mut it, "address_space")?;
    let address_space = std::str::from_utf8(&source.as_bytes()[address_space_node.byte_range()])
        .unwrap()
        .try_into()?;

    let mut access_mode = None;
    if let Some(n) = it.next() {
        if n.kind() == "comma" {
            let access_mode_node = parse_node(&mut it, "access_mode")?;
            access_mode = Some(
                std::str::from_utf8(&source.as_bytes()[access_mode_node.byte_range()])
                    .unwrap()
                    .try_into()?,
            );
        }
    }

    Ok((Some(address_space), access_mode))
}

fn parse_attributes(node: Node, source: &str) -> Result<Attributes> {
    let mut group = None;
    let mut binding = None;
    for child in node.children(&mut node.walk()) {
        if child.kind() != "attribute" {
            continue;
        }

        let mut cursor = child.walk();
        let mut attribute_children_iterator = child.children(&mut cursor);
        eat(&mut attribute_children_iterator, "attr")?;
        let identifier_node = parse_node(&mut attribute_children_iterator, "ident")?;
        eat(&mut attribute_children_iterator, "paren_left")?;
        let value_node = parse_node(&mut attribute_children_iterator, "literal_or_ident")?;
        eat(&mut attribute_children_iterator, "paren_right")?;

        let identifier =
            std::str::from_utf8(&source.as_bytes()[identifier_node.byte_range()]).unwrap();
        let value = std::str::from_utf8(&source.as_bytes()[value_node.byte_range()]).unwrap();

        match identifier {
            "group" => group = Some(u32::from_str(value).unwrap()),
            "binding" => binding = Some(u32::from_str(value).unwrap()),
            _ => {}
        }
    }
    Ok(Attributes {
        group: group.ok_or(GraphicsError::WgslMissingAttribute)?,
        binding: binding.ok_or(GraphicsError::WgslMissingAttribute)?,
    })
}

fn eat<'a>(
    iterator: &mut impl ExactSizeIterator<Item = Node<'a>>,
    expected: &'static str,
) -> Result<()> {
    let next = iterator.next();
    match next {
        Some(next) if next.kind() == expected => Ok(()),
        next => Err(GraphicsError::WgslParseError {
            expected,
            encountered: next.map(|n| n.kind()),
        }),
    }
}

fn parse_node<'a>(
    iterator: &mut impl ExactSizeIterator<Item = Node<'a>>,
    expected: &'static str,
) -> Result<Node<'a>> {
    let next = iterator.next();
    match next {
        Some(next) if next.kind() == expected => Ok(next),
        next => Err(GraphicsError::WgslParseError {
            expected,
            encountered: next.map(|n| n.kind()),
        }),
    }
}
fn parse_optional_node<'a>(
    iterator: &mut Peekable<impl ExactSizeIterator<Item = Node<'a>>>,
    expected: &'static str,
) -> Option<Node<'a>> {
    match iterator.peek() {
        Some(next) if next.kind() != expected => return None,
        None => return None,
        _ => {}
    };

    let next = iterator.next();
    match next {
        Some(next) if next.kind() == expected => Some(next),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_shader_metadata() -> crate::Result<()> {
        let variables = super::extract_global_variables_from_shader_source(
            r"
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> mesh_uniform: MeshUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var t_sampler: sampler;

@group(3) @binding(0)
var<storage, read> light_storage: LightStorage;

            ",
        )?;

        assert_eq!(variables.len(), 5);

        let camera_variable = &variables[0];
        assert_eq!(camera_variable.attributes.group, 0);
        assert_eq!(camera_variable.attributes.binding, 0);
        assert_eq!(camera_variable.identifier, "camera");
        assert_eq!(
            camera_variable.address_space.unwrap(),
            AddressSpace::Uniform
        );
        assert!(camera_variable.access_mode.is_none());
        assert_eq!(camera_variable.kind, VariableKind::Other);

        let mesh_uniform = &variables[1];
        assert_eq!(mesh_uniform.attributes.group, 1);
        assert_eq!(mesh_uniform.attributes.binding, 0);
        assert_eq!(mesh_uniform.identifier, "mesh_uniform");
        assert_eq!(mesh_uniform.address_space.unwrap(), AddressSpace::Uniform);
        assert!(mesh_uniform.access_mode.is_none());
        assert_eq!(mesh_uniform.kind, VariableKind::Other);

        let t_diffuse = &variables[2];
        assert_eq!(t_diffuse.attributes.group, 2);
        assert_eq!(t_diffuse.attributes.binding, 0);
        assert_eq!(t_diffuse.identifier, "t_diffuse");
        assert!(t_diffuse.address_space.is_none());
        assert!(t_diffuse.access_mode.is_none());
        assert_eq!(t_diffuse.kind, VariableKind::Texture);

        let t_sampler = &variables[3];
        assert_eq!(t_sampler.attributes.group, 2);
        assert_eq!(t_sampler.attributes.binding, 1);
        assert_eq!(t_sampler.identifier, "t_sampler");
        assert!(t_sampler.address_space.is_none());
        assert!(t_sampler.access_mode.is_none());
        assert_eq!(t_sampler.kind, VariableKind::Sampler);

        let light_storage = &variables[4];
        assert_eq!(light_storage.attributes.group, 3);
        assert_eq!(light_storage.attributes.binding, 0);
        assert_eq!(light_storage.identifier, "light_storage");
        assert_eq!(light_storage.address_space, Some(AddressSpace::Storage));
        assert_eq!(light_storage.access_mode, Some(AccessMode::Read));
        assert_eq!(light_storage.kind, VariableKind::Other);
        Ok(())
    }
}
