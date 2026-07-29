"""Design A: two uniffi namespaces in one cdylib, objects passed across."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out"))
from python import core_lib, ext_lib

ep1, ep2 = core_lib.Endpoint(), core_lib.Endpoint()
p = ext_lib.Ping()
print("ep1.id =", ep1.id(), "| ep2.id =", ep2.id())
print("ping(ep1) ->", p.ping(ep1))
assert (ep1.id(), ep2.id()) == (0, 1), "global counter not shared"
print("OK: cross-namespace object passing works, single shared global")
