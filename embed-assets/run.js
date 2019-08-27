
const assert = require('assert');

module.exports = function (Elm, output) {
    const { ports: portSubscriptions = {}, flags } = output;
    const app = Elm.Main.init(flags !== undefined ? { flags } : undefined);
    const subscriptionCounts = {};

    function send(portName, data) {
        app.ports[portName].send(data);
    }

    if (app.ports !== undefined) {
        for (let portName of Object.keys(app.ports)) {
            if (app.ports[portName].subscribe !== undefined) {
                subscriptionCounts[portName] = 0;
                app.ports[portName].subscribe(data => {
                    const index = subscriptionCounts[portName];
                    subscriptionCounts[portName] += 1;
                    assert(
                        Array.isArray(portSubscriptions[portName]),
                        `port "${portName}" has been called by elm but should never be.`,
                    );
                    const expectedNumberOfMsgs = portSubscriptions[portName].length;
                    assert(
                        index < expectedNumberOfMsgs,
                        `port "${portName}" has been called ${subscriptionCounts[portName]} times by elm but should only have been called ${expectedNumberOfMsgs} times.`,
                    );
                    assert.deepStrictEqual(
                        data,
                        portSubscriptions[portName][index],
                        `Wrong data sent to port ${portName} on occurance ${index}`,
                    )
                })
            }
        }
    }

    process.on('exit', () => {
        if (app.ports !== undefined) {
            for (let portName of Object.keys(app.ports)) {
                const expectedNumberOfMsgs = portSubscriptions[portName] !== undefined ? portSubscriptions[portName].length : 0;
                assert(
                    subscriptionCounts[portName] == expectedNumberOfMsgs,
                    `port ${portName} has been called ${subscriptionCounts[portName]} times but should be called exactly ${expectedNumberOfMsgs} times.`,
                );
            }
        }
    });

}