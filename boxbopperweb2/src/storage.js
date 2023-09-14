export default {
	getBestScore: function(levelTitle) {
		var x = parseInt(localStorage.getItem('bxbop_bestScore_'+levelTitle));
		if(isNaN(x)) return 'unsolved';
		return x;
	},
	setBestScore: function(levelTitle, bestScore) {
		localStorage.setItem('bxbop_bestScore_'+levelTitle, bestScore);
	},
}
