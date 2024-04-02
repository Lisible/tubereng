![tuber logo](img/tuber_logo.png)

# tuber

*Make your games taste like a Piemontese salad*

***tuber*** is a game engine I'm making. 

It is built on top of `wgpu` on the graphics side, it has its own Entity
Component System library, of which the API is highly inspired by 
[bevy](https://github.com/bevyengine/bevy)'s one, although the underlying
implementation is different (`tubereng-ecs` is not archetype-based).

It is being built for myself and I discourage using it for anything serious
for now.

This is meant to be a personal, mostly educational, side-project but PRs are
still welcomed and will be reviewed by myself.

# How to run the examples

You can run the examples by running the following command
```bash
cargo run -p <example>
```
