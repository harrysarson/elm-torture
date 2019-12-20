module Main exposing (main)

{-| This is not how Cmd.map is designed to be used.
However, it should test some of the knarly bits of elm/core:Platform
-}

import Platform
import Process
import Task
import Util.Cmds


type Msg
    = Init


init : ( (), Cmd Msg )
init =
    ( ()
    , Task.perform (\() -> Init) (Process.sleep 1)
    )


update : Msg -> () -> ( (), Cmd Msg )
update msg () =
    case msg of
        Init ->
            ( ()
            , Util.Cmds.write "done"
            )


main : Platform.Program () () Msg
main =
    Platform.worker
        { init = \() -> init
        , update = update
        , subscriptions = \_ -> Sub.none
        }
