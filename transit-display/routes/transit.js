var child           = require('child_process');
var q               = require('q');
var express         = require('express');
var router          = express.Router();

function executeTransit(repo, from, to) {
    var deferred = q.defer();
    var cmd = '../target/transit';
    var args = [ '--json', repo ];
    if( from && to) {
        args[1] = from;
        args[2] = to;
    }

    var transitOutput = "";
    var transitErr    = "";

    var kinderProcess = child.spawn(cmd, args);
    kinderProcess.stdout.on('data', function (data) {
            process.stdout.write(data.toString('utf8'));
            transitOutput += data;
        });

    kinderProcess.stderr.on('data', function (data) {
            console.log('stderr: ' + data.toString('utf8'));
            transitErr += data;
        });

    kinderProcess.on('close', function (code) {
            console.log('child process exited with code ' + code);
            if (code === 0) {
                console.log("transit ok");

                console.log("transit:", transitOutput);
                var parsed = JSON.parse(transitOutput);
                console.log("PARSED")
                deferred.resolve(parsed);

            } else {
                console.log("ERROR: transit execution failed!");
                deferred.reject({msg: transitErr, code: code})
            }
        });

    kinderProcess.on('error', function (error) {
            console.log("error spawning child process:", error);
            deferred.reject({msg: 'transit (' + cmd + ') executable failed to run.', code: error})
        });
    return deferred.promise;
}

/* GET transit page. */
router.get('/', function (req, res, next) {
    var diffs;

    console.log("req:", req.query);
    /* example output
     * struct Output {
     *    old_commit: Oid,
     *    new_commit: Oid,
     *    old_filename: String,
     *    new_filename: String,
     *    origin_line: u32,
     *    destination_line: u32,
     *    num_lines: u32,
     * }   */
    var repositoryName = req.query.repo ? req.query.repo : '';
    var repoPath = req.query.repopath ? req.query.repopath : '';
    var oldCommit = req.query.oldcommit ? req.query.oldcommit : '';
    var newCommit = req.query.newcommit ? req.query.newcommit : '';

    var mockDataDiffs = [
        {
            old_commit: 'Oid',
            new_commit: 'Oid',
            old_filename: 'String',
            new_filename: 'String',
            origin_line: 'u32',
            destination_line: 'u32',
            num_lines: 'u32'
        },
        {
            old_commit: 'Oid2',
            new_commit: 'Oid2',
            old_filename: 'String2',
            new_filename: 'String2',
            origin_line: 'u322',
            destination_line: 'u322',
            num_lines: 'u322'
        }
    ];

    if(repositoryName && repoPath) {
        console.log('calling transit with:', repoPath, oldCommit, newCommit);

        executeTransit(repoPath, oldCommit ? oldCommit: null, newCommit ? newCommit : null)
            .then(function(diffs) {
                renderOutput({
                    title: 'transit express',
                    repository: repositoryName,
                    diffs: diffs
                });
            })
            .catch(function inCaseOfFailure(message) {
                res.render('error', { message: message });
            });
    } else {
        //test mock up data
        repositoryName = 'test mock up data';
        diffs = mockDataDiffs;
        renderOutput({
            title: 'transit express',
            repository: repositoryName,
            diffs: diffs
        });
    }

    function renderOutput(output) {
        console.log("DONE EXECUTING AND IS FINE");
        res.render('transit', output);
    }
});


module.exports = router;
