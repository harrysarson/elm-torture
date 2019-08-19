port module Util.Programs exposing (print)

import Platform


port write : String -> Cmd never


print : String -> Platform.Program () () a
print string =
    Platform.worker
        { init = \() -> ( (), write string )
        , update = \_ () -> ( (), Cmd.none )
        , subscriptions = \() -> Sub.none
        }
