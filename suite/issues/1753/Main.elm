module Main exposing (main)

import Array exposing (Array)
import Browser
import Platform
import Util.Programs


type Msg
    = Msg (Array ())


main : Program () () Msg
main =
    Util.Programs.noop
