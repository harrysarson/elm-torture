
const assert = require('assert');

module.exports = function (Elm, output) {
    const { ports = [], flags } = output;
    const app = Elm.Main.init(flags !== undefined ? { flags } : undefined);
    let portEventIndex = 0;

    function send(portName, data) {
        app.ports[portName].send(data);
    }

    if (app.ports !== undefined) {
        for (const portName of Object.keys(app.ports)) {
            if (app.ports[portName].subscribe !== undefined) {
                app.ports[portName].subscribe(data => {
                    assert(
                        portEventIndex < ports.length,
                        `There should be exactly "${ports.length}" port events but this is event ${portEventIndex + 1}.`,
                    );
                    const [type, expectedName, expectedData] = ports[portEventIndex];
                    assert(
                        type === "command",
                        `Port event ${portEventIndex} should be a ${type} but command ${portName} received (with value ${data}).`,
                    );
                    assert(
                        expectedName === portName,
                        `Port event ${portEventIndex} should be command ${expectedName} but command ${portName} received (with value ${data}).`,
                    );
                    assert.deepStrictEqual(
                        data,
                        expectedData,
                        `Wrong data sent to port ${portName} during port event ${portEventIndex + 1}`,
                    )
                    portEventIndex += 1;
                })
            }
        }
    }

    process.on('exit', () => {
        if (app.ports !== undefined) {
            assert(
                portEventIndex == ports.length,
                `There have been ${portEventIndex} port events but should have been exactly ${ports.length} port events.`,
            );
        }
    });

}
