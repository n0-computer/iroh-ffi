# iroh-ffi (Archived)

> **This repository is archived and provided as a reference example only.**

For up-to-date guidance on using iroh in other languages, see the official documentation:

---

## Using Iroh in Other Languages

While iroh is written in Rust, it can be used in many other languages and environments. There are several practical approaches to using iroh in your language of choice.

### Write a Wrapper

If you're comfortable with a little bit of Rust, write your own small wrapper around iroh that covers just what you need and exposes your application specific functionality over a local http server or daemon. This approach:

- Gives you full control over the functionality you expose
- Requires minimal Rust knowledge beyond basic CLI patterns
- Can be called from any language

Check out [sendme](https://github.com/n0-computer/sendme), [callme](https://github.com/n0-computer/callme), and [dumbpipe](https://github.com/n0-computer/dumbpipe) as examples.

### Build Your Own FFI Wrapper

Write your own FFI wrapper from Rust to your target language (Python, Go, etc.) that covers just what you need from the iroh API and protocols. This gives you:

- Complete control over the API surface
- The ability to tailor it to your specific use case
- Type-safe bindings for your language

Reference this repository (iroh-ffi) for patterns and examples.

### Community Bindings

The community has built language bindings that are open source and available for use. For the full list, see the official documentation: **[Using Iroh in Other Languages](https://docs.iroh.computer/deployment/other-languages)**

### Professional Bindings Support

The number0 engineering team can help you build and maintain production-grade language-specific bindings. [Contact us](https://iroh.computer/services/support) to discuss your requirements.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
