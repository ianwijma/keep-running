const Yargs = require('yargs');
const {spawn} = require("child_process");

const SECONDS_IN_A_MINUTE = 60;
const SECONDS_IN_A_HOUR = 60 * 60;

const yargs = Yargs(process.argv.splice(2))
    .scriptName('kr')
    .usage('kr [args] <thing-to-run>')
    .option('rpm', {
        default: 0,
        description: 'The amount of Retries Per Minute, before we stop retrying',
        type: 'number'
    })
    .option('rph', {
        default: 0,
        description: 'The amount of Retries Per Hour, before we stop retrying',
        type: 'number'
    })
    .help();

const { _ = [], rpm, rph } = yargs.argv;
const [command, ...args  ] = _;

if (!command) {
    return yargs.showHelp();
}

let historyMax = 4;
let seconds = SECONDS_IN_A_MINUTE;
let restartName = 'minute';

if (rpm && rph) {
    return console.error('Currently, can not define both --rpm and --rph, please choose only one.');
} else if (rpm) {
    historyMax = rpm
} else if (rph) {
    historyMax = rph
    seconds = SECONDS_IN_A_HOUR
    restartName = 'hour'
}

/**
 * @type {Object<number, string>}
 */
const history = {};

/**
 * @param {string} logs
 */
const pushHistory = (logs) => history[(new Date).getTime()] = logs;

const updateHistory = () => {
    const clearKeys = [];

    const maxAge = (new Date).getTime() + seconds;
    for (const historyTime in history) {
        if (historyTime > maxAge) {
            clearKeys.push(historyTime)
        }
    }

    clearKeys.forEach((key) => delete history[key]);
}

const checkHistory = () => Object.keys(history).length <= historyMax;

const runCommand = () => {
    console.log(`Running ${command} ${args.join(' ')}`);
    const runner = spawn(command, args);

    const commandLogs = [];

    const pushLog = (logEntry) => commandLogs.push(logEntry);

    const trimLog = () => {
        if(commandLogs.length > 5000) commandLogs.shift()
    };

    const clearLogs = () => commandLogs.length = 0;

    const getLogs = () => commandLogs.join('\n');

    const handleOut = (type, message) => {
        pushLog(`[${type}]\t${message}`);
        trimLog();
    };

    const handleExit = (exitCode) => {
        console.log(`[CRASH] exit code: ${exitCode}`)
        if (exitCode !== 0) {
            pushHistory(getLogs());
            clearLogs();
            updateHistory();
            if (checkHistory()) {
                console.log('Restarting...');
                runCommand();
            } else {
                console.error(`The process crashed more then ${historyMax} times in the past ${restartName}, stop retrying.`);
                console.error(`See below the past ${historyMax} crashes:`);
                for (const time in history) {
                    console.error(`Crash @ ${time}:\n${history[time]}`);
                }
            }
        }
    }

    runner.on('close', (exitCode) => handleExit(exitCode))
    runner.stdout.on('data', (data) => console.log(data.toString().trim()))
    runner.stdout.on('data', (data) => handleOut('stdout', data.toString().trim()))
    runner.stderr.on('data', (data) => handleOut('stderr', data.toString().trim()))
}

// Command goes BRRRR
runCommand();
