module Main exposing (main)

import Platform
import Time
import Util.Cmds


type Msg
    = Init
    | Time1
    | Time2


type Model
    = One
    | Two
    | Three


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Init ->
            ( Two
            , Cmd.none
            )

        Time1 ->
            ( Two
            , Util.Cmds.write "1"
            )

        Time2 ->
            ( Three
            , Util.Cmds.write "2"
            )


subscriptions : Model -> Sub Msg
subscriptions model =
    case model of
        -- Get app in sync with 300ms intervals
        One ->
            Time.every 300 (\_ -> Init)

        Two ->
            Sub.batch
                [ Time.every 30 (\_ -> Time1)
                , Time.every 100 (\_ -> Time2)
                ]

        Three ->
            Sub.none


main : Platform.Program () Model Msg
main =
    Platform.worker
        { init = \() -> ( One, Cmd.none )
        , update = update
        , subscriptions = subscriptions
        }
