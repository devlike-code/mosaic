# TODO

[M] properties layer for comfort functions
    [ ] `get_property_map` returns all components and their data
[N] memory-work with getters/setters for data
    [ ] rename empty field (for alias) into "self"
    [ ] working with bricks through a `BrickEditor` interface with a `get_field` and `set_field` structure
    [ ] the `BrickEditor` should send update events to the system either directly or through a `.commit(self)`
    [ ] create a method in brick to get a builder: `Brick::edit(&mut self) -> BrickEditor`
[M?] string allocation and retrieval
    [ ] change `string` type in datatypes to `blob` (256-byte blob)
    [ ] make a `String` component that is aliasing a `blob` (or `b256`)
    [ ] make a `string` layer with functions:
        [ ] `create_string(str: &str) -> EntityId` that makes an object and its properties and 
            returns a EID type that's actually the hash of the string
        [ ] `string_exists(str: &str) -> bool` to help check whether a string already exists by hashing it
        [ ] `get_string(e: EntityId) -> String` that gets an object and attaches all the properties into one string
        [ ] `update_string(e: EntityId, str: &str)` so that it updates the string inside and returns the same ID
        [ ] `delete_string(e: EntityId)` that deletes the object and all properties

                Example: 
                    let e = create_object() -> EntityId
                    add_outgoing_property(e, ...)
                    return e

                    1 1 1 Object ""
                    2 1 2 String " qweqweqweqwdasdqweqwewqdqwdqw.q"
                    3 1 3 String " qwe qweqwe qwe qwe qwe.\0\0\0\0"

[ ] top-level functionality
    [ ] wasm
    [ ] c#
    [ ] c++
[ ] live debugging
[ ] graph functionality layer
    [ ] reachability
    [ ] adjacency
    [ ] DFS & BFS
    [ ] graph spanning
[ ] transformer (Transform > ...)
    [ ] graph match
    [ ] fsm
    [ ] calculator
