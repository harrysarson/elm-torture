module Util.Programs exposing (noop, print)

import Platform
import Util.Cmds


print : String -> Platform.Program () () a
print string =
    Platform.worker
        { init = \() -> ( (), Util.Cmds.write string )
        , update = \_ () -> ( (), Cmd.none )
        , subscriptions = \() -> Sub.none
        }


noop : Platform.Program () () a
noop =
    Platform.worker
        { init = \() -> ( (), Cmd.none )
        , update = \_ () -> ( (), Cmd.none )
        , subscriptions = \() -> Sub.none
        }
