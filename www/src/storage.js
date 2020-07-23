export default {
	getBestScore: function(levelnum) {
		var x = parseInt(localStorage.getItem('bxbop_bestScore_'+levelnum));
		if(isNaN(x)) return 'unsolved';
		return x;
	},
	setBestScore: function(levelnum,bestScore) {
		localStorage.setItem('bxbop_bestScore_'+levelnum, bestScore);
	},
}
