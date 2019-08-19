module Main exposing (test2)

import Platform
import Util.Programs
import Module1

test2 : ()
test2 =
    Module1.test1

main : Platform.Program () () ()
main =
    Util.Programs.print "ok"
