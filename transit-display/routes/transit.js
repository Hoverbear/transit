var express = require('express');
var router = express.Router();

/* GET transit page. */
router.get('/', function(req, res, next) {
    // TODO: get data to display
    // TODO: format data into the output object
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
    var output = { 
        title: 'transit express',
        repository: 'test mock up data',
        diffs: [
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
        ]
    };
    res.render('transit', output);
});

module.exports = router;
