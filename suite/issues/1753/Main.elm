module Main exposing (main)

import Browser
import Array exposing (Array)
import Platform
import Util.Programs

type Msg
    = Msg (Array ())


main : Program () () Msg
main =
    Util.Programs.print "ok"