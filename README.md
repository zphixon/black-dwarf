# black dwarf

*a build system for C from the end of the universe*

What was once a brightly shining star burns no longer. It has ceased to fuse
atoms and the last of its life-giving energy has radiated away into space. All
that remains is the cold, hard core.

We're talking about C, of course.

## tests

Tests are structured like so:

- tests/ - Well-formed black dwarf configs
  - should_fail/ - Incorred black dwarf configs
  - toml/ - Well-formed TOML parser tests
    - should_fail/ - Incorrect TOML files

Every test outside of the tests/toml/should_fail/ directory should have lines
prefixed with `#--` containing the debug print output (`{:#?}`) of the TOML
`Value`. E.g

```toml
#--{
#--    "a": "b",
#--}
a = 'b'
```

Tests in the top level tests/ directory should also have lines prefixed with
`#==` containing the debug print output of the `BlackDwarf` config.

---

This is a shitpost

# I hate autotools I hate make I hate kconfig I hate waf I hate scons I hate cmake I hate meson I hate ninja I hate xmake I hate gmake I hate SLNs I hate msbuild I hate tup I hate qmake I hate bazel I hate boost.build I hate conan
