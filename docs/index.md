# Where is seal now

Seal is currently just a toy. Its not perfect and not even close to something
productin ready. I have planned to redesign seal again this is the actually the
second time iam doing this. But this time iam sure i will create a code base
which both scalable and maintainable.


# Checklist of Changes 

- [x] transition to a library based project.
- [x] update the lexer to use an iterator based approch.
- [x] improve the parser so that it returns an AST.
- [ ] redesign the core execution path to a central api in lib.
- [ ] build a builtins registary where builtins can be dynamically added.
- [ ] after the registary implement builtin trait for luabuiltin 
- [ ] finally you will have a plugin echo system.


# The vision

The vision is to transition into a more modular codebase, and being able to 
make the changes as i wish. Ideally i should be able to maintain this codebase
without removing current code but by adding new code. every piece should be
independant and testable.

