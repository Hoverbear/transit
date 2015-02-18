var child           = require('child_process');
var q               = require('q');
var express         = require('express');
var router          = express.Router();

function executeTransit(repo, from, to) {
    var deferred = q.defer();
    var cmd = '../target/transit';
    var args = [ repo ];
    if( from && to) {
        args[1] = from;
        args[2] = to;
    }
    var kinderProcess = child.spawn(cmd, args);
    var fileContents;
    var transitOutput = "";

    kinderProcess.stdout.on('data', function (data) {
        process.stdout.write(data.toString('utf8'));
        transitOutput += data;
    });

    kinderProcess.stderr.on('data', function (data) {
        console.log('stderr: ' + data.toString('utf8'));
    });

    kinderProcess.on('close', function (code) {
        console.log('child process exited with code ' + code);
        if (code === 0) {
            console.log("transit ok");

            console.log("transit:", transitOutput);
            deferred.resolve(JSON.parse(transitOutput));
            // load file contents
            //fs.readFile(calendarFileName + postfix, {encoding: 'utf8'}, function(err, data) {
            //    if (err) {
            //        log.error(err);
            //        deferred.reject(err);
            //    } else {
            //        fileContents = data;
            //        log.info("calendar generation succeeded!");
            //        deferred.resolve({contents: fileContents, fileName: calendarFileName + postfix});
            //    }
            //});

        } else {
            console.log("ERROR: transit execution failed!");
            deferred.reject({msg: "transit execution failed!", code: code})
        }
    });
    return deferred.promise;
}

/* GET transit page. */
router.get('/', function (req, res, next) {
    var diffs;
    // TODO: get data to display
    // TODO: format data into the output object
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
            }, function inCaseOfFailure(message) {
                renderOutput({
                    title: 'transit express',
                    repository: repositoryName + 'failed to read repository: ' + repoPath + ' due to: ' + message,
                    diffs: []
                });
                
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
        res.render('transit', output);
    }
});


module.exports = router;
