var express         = require('express');
var router          = express.Router();


/* GET transit page. */
router.get('/', function (req, res, next) {
    
    renderOutput({
        title: 'transit express'
    });

    function renderOutput(output) {
        res.render('select', output);
    }
});


module.exports = router;
