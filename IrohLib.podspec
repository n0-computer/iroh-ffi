Pod::Spec.new do |spec|
  spec.name         = "IrohLib"
  spec.version      = "0.27.0"
  spec.summary      = "iroh is a toolkit for building distributed apps"
  spec.description  = <<-DESC
                      Build distributed apps that raise the status quo for your users.
                      Open source and universally compatible, it's the quickest route from
                      concept to connected devices.
                    DESC
  spec.homepage           = "https://iroh.computer"
  spec.license            = { :type => "MIT & Apache License, Version 2.0",
                              :text => <<-LICENSE
                                Refer to LICENSE-MIT and LICENSE-APACHE in the repository.
                              LICENSE
                            }
  spec.author             = { "b5" => "sparkle_pony_2000@n0.computer" }
  spec.social_media_url   = "https://twitter.com/iroh_n0"
  spec.source             = { :git => "https://github.com/n0-computer/iroh-ffi.git", :tag => "#{spec.version}" }
  spec.static_framework   = true
  spec.source_files       = "IrohLib/Sources/IrohLib/*.swift"
  spec.swift_version      = '5.9'
  spec.framework          = "SystemConfiguration"
  spec.ios.deployment_target  = '15.0'
  spec.dependency 'IrohLibFramework', "#{spec.version}"
end
