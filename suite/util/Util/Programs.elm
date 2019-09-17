module Util.Programs exposing (noop, print, sendCmd)

import Platform
import Util.Cmds


sendCmd : Cmd a -> Platform.Program () () a
sendCmd cmd =
    Platform.worker
        { init = \() -> ( (), cmd )
        , update = \_ () -> ( (), Cmd.none )
        , subscriptions = \() -> Sub.none
        }


print : String -> Platform.Program () () a
print string =
    sendCmd (Util.Cmds.write string)


noop : Platform.Program () () a
noop =
    sendCmd Cmd.none
