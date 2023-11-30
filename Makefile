DEPS:=iroh.h iroh.go iroh.a

export CARGO_TARGET_DIR=target

all: $(DEPS)
.PHONY: all

# Create a file so that parallel make doesn't call `./install-filcrypto` for
# each of the deps
$(DEPS): .install-iroh  ;

.install-filcrypto: rust
	go clean -cache -testcache
	./install-filcrypto
	@touch $@

clean:
	go clean -cache -testcache
	rm -rf $(DEPS) .install-iroh
	rm -f ./runner
	cd rust && cargo clean && cd ..
.PHONY: clean
