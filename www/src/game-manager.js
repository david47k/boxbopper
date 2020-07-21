import { Game, Vector, Level, Move, Object, load_builtin } from 'wasm-game';
import CONFIG from './config';
import { View } from './view';
import { Controller } from './controller';
import Storage from './storage';

export class GameManager {
	constructor() {
		this.levelNumber = 0;
		this.baseLevel = load_builtin(this.levelNumber);
		this.restart();
		this.view = new View(
			this.game,
			this.render.bind(this)
		);
		this.controller = new Controller();
	}

	restart() {
		this.game = new Game(this.baseLevel);
		console.log(this.game);
		this.lastUpdate = undefined;
	}

	render() {
		this.view.render(this.game);
	}

	tick() {
		const lastUpdate = Date.now();
		if(this.lastUpdate) {
			this.game.process(lastUpdate - this.lastUpdate, this.controller.movement);
/*			if(this.game.is_over()) {
				this.restart();
				return;
			}				*/
/*			if(this.game.score > Storage.getBestScore()) {
				//localStorage.setItem('bestScore',this.game.score);
				Storage.setBestScore(this.game.score);
			} */
		}
		this.lastUpdate = lastUpdate;
		this.render();
	}
	
	run() {
		setInterval(this.tick.bind(this), 1000 / CONFIG.FPS);
	}
	
}
