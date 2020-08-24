import { Game, Vector, Level, Move, Obj, load_builtin } from 'wasm-game';
import CONFIG from './config';
import { View } from './view';
import { Controller } from './controller';
import Storage from './storage';

export class GameManager {
	constructor() {
		this.levelNumber = 0;
		this.transitionList = [];
		this.restart(this.levelNumber);
		document.getElementById('prev_button').disabled = (this.levelNumber==0);
		this.view = new View(this.game.get_level_width(),this.game.get_level_height());
		this.controller = new Controller();

		document.getElementById("prev_button").addEventListener('click', function() {
			document.gameManager.prevLevel();
		});
		document.getElementById("next_button").addEventListener('click', function() {
			document.gameManager.nextLevel();
		});
		document.getElementById("retry_button").addEventListener('click', function() {
			document.gameManager.restart(document.gameManager.levelNumber);
		});
		
		this.view.container.addEventListener('click', function(ev) {
			var gm = document.gameManager;
			var x = Math.floor(ev.offsetX / gm.view.unitOnScreen);
			var y = Math.floor(ev.offsetY / gm.view.unitOnScreen);
			// if in same y as human, apply L/R to try and make x match
			var diff = 0;
			var m;
			if(gm.game.human_pos.as_array()[1] == y) {
				diff = x - gm.game.human_pos.as_array()[0];
				console.log("x: ", x, " y: ", y);

				if(diff > 0) {
					m = 1;
				} else if(diff < 0) {
					m = 3;
					diff = Math.abs(diff);
				}
			} else if(gm.game.human_pos.as_array()[0] == x) {
				diff = y - gm.game.human_pos.as_array()[1];
				if(diff > 0) {
					m = 2;
				} else if(diff < 0) {
					m = 0;
					diff = Math.abs(diff);
				}
			}
			if(diff <= 0 || diff > 50) return; // no clear direction or something screwy
			for(var i=0;i<diff;i++) {
				gm.game.append_move_js(m);
			}
		});
		document.gameManager = this;			// need to persist the object in the document, our callbacks aren't getting called with correct this
	}

	restart(levelNum) {	
		this.game = new Game(this.levelNumber);
		this.levelTitle = this.game.get_level_title();
		this.bestScore = Storage.getBestScore(this.levelTitle);
		if(this.view) {
			this.view.setUp(this.game.get_level_width(),this.game.get_level_height());
			document.getElementById('prev_button').disabled = (levelNum==0);
			document.getElementById('next_button').disabled = (levelNum==this.game.get_max_level_number());
		}
	}

	nextLevel() {
		if(this.levelNumber < this.game.get_max_level_number()) this.levelNumber += 1;
		this.restart(this.levelNumber);
	}

	prevLevel() {
		this.levelNumber -= 1;
		if(this.levelNumber < 0) this.levelNumber = 0;
		this.restart(this.levelNumber);
	}

	render() {
		var gm = document.gameManager;
		if(gm.game.have_win_condition()) {
			if(isNaN(parseInt(Storage.getBestScore(gm.levelTitle))) || gm.game.num_moves < Storage.getBestScore(gm.levelTitle)) {
				gm.bestScore = gm.game.num_moves;
				Storage.setBestScore(gm.levelTitle, gm.bestScore);
			}
		}
		if(gm.game) {
			gm.game.process_moves_js();
			gm.view.render(gm.game, gm.game.human_pos);
		}
	}

	runOnTimer() {
		setInterval(this.render, 1000/CONFIG.FPS);
		this.render();
	}
	
}
