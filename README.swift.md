# Iroh Swift

## Xcode and IOS

- Run `make_swift.sh`.
- Add `IrohLib` as a local package dependency under `Frameworks, Libraries, and Embedded Content` in the `General` settings of your project.
- Run `Build`
- Check that it is now listed under `Frameworks, Libraries, and Embedded Content`, if not click `+` again and add it from the list.
- Add `SystemConfiguration` as a Framework.
- Now you can just import the library in Swift with a standard import statement like `import IrohLib`.
