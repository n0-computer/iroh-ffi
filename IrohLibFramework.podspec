Pod::Spec.new do |spec|
  spec.name         = "IrohLibFramework"
  spec.version      = "0.21.0-1"
  spec.summary      = "Complied rust cocoa framework for Iroh"
  spec.description  = <<-DESC
                   Complied rust cocoa framework for Iroh.
                   DESC
  spec.homepage     = "https://github.com/n0-computer/iroh-ffi"
  spec.license      = { :type => "MIT & Apache License, Version 2.0",   :text => <<-LICENSE
                          Refer to LICENSE-MIT and LICENSE-APACHE in the repository.
                        LICENSE
                      }
  spec.author       = { "b5" => "sparkle_pony_2000@n0.computer" }
  spec.ios.deployment_target  = '15.0'
  spec.static_framework = true
  # WARNING: we're publishing v0.21.0-1, not v0.21.0 because of an error in the binary artifact of the original release
  # in v0.21.1 or higher, switch back to the template-driven URL:
  # spec.source = { :http => "https://github.com/n0-computer/iroh-ffi/releases/download/v#{spec.version}/IrohLib.xcframework.zip" }
  spec.source = { :http => "https://github.com/n0-computer/iroh-ffi/releases/download/v0.21.0/IrohLib.xcframework.zip" }
  spec.vendored_frameworks = 'Iroh.xcframework'
end
