module Module1 exposing (test1)

import Util.Programs
import Platform

test1 : ()
test1 =
    ()

main : Platform.Program () () ()
main =
    Util.Programs.print "ok"
