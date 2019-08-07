port module Main exposing (main)

import Platform

port write: String -> Cmd never

main: Platform.Program () () ()
main =
    Platform.worker
        { init = \() -> ((), write "Hello World!")
        , update = \() () -> ((), Cmd.none)
        , subscriptions = \() -> Sub.none
        }