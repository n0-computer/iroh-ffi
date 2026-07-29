"""Design B, minimal (no iroh): two .so files, one shared copy of the core."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out"))
from p2.python import p2_core_lib, p2_ext_lib

ep1, ep2 = p2_core_lib.Endpoint(), p2_core_lib.Endpoint()
p = p2_ext_lib.Ping()
print("ep1.id =", ep1.id(), "| global @", hex(ep1.global_addr()))
print("ping(ep1) ->", p.ping(ep1))
assert (ep1.id(), ep2.id()) == (0, 1), "core statics NOT shared between the two .so files"
print("OK: two .so files, one shared copy of the core crate")
