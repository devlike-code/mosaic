# TODO

[M] properties layer for comfort functions
    [x] `get_property_map` returns all components and their data
        - called `get_entity_archetype`, returns a vector of bricks that adorn the entity directly or not
[N] memory-work with getters/setters for data
    [x] rename empty field (for alias) into "self"
    [...] working with bricks through a `BrickEditor` interface with a `get_field` and `set_field` structure
    [ ] the `BrickEditor` should send update events to the system either directly or through a `.commit(self)`
    [x] create a method in brick to get a builder: `Brick::edit(&mut self) -> BrickEditor`
[M] string allocation and retrieval
    [x] change `string` type in datatypes to `blob` (256-byte blob)
        - renamed `str` to `b256` and changed bytesize to 32
    [x] make a `String` component that is aliasing a `blob` (or `b256`)
    [x] make a `string` layer with functions:
        [x] `create_string(str: &str) -> EntityId` that makes an object and its properties and 
            returns a EID type that's actually the hash of the string
        [x] `string_exists(str: &str) -> bool` to help check whether a string already exists by hashing it
        [x] `get_string(e: EntityId) -> String` that gets an object and attaches all the properties into one string
            - called `recover_string`, works like a charm
        [x] `update_string(e: EntityId, str: &str)` so that it updates the string inside and returns the same ID
            - this is not needed as strings are interned and are never updated - the identifiers are changed in the brick
        [x] `delete_string(e: EntityId)` that deletes the object and all properties
            - not deleting all properties, as this is going to be a systemic thing triggered through the bus

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
