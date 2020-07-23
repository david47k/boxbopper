
//import { Move } from "wasm-game";

export class Controller {
	constructor() {
		window.addEventListener('keydown', (ev) => {
			var which = ev.which;
			if(which == 82) { // R
				document.gameManager.restart(document.gameManager.levelNumber);
			} else if(which == 78) { // N
				document.gameManager.nextLevel();
			} else if(which == 80) { // P
				document.gameManager.prevLevel();
			} else if((which == 32 || which == 13) && document.gameManager.game.have_win_condition()) { // space or enter
				document.gameManager.nextLevel();
			} else {
				document.gameManager.game.process_keys([which]);
			}
		});
		window.addEventListener('keyup', ({ which }) => {
				// we can only consider a keyup if it applys to the current keydown that we have
				// this.movement = undefined;
		});
	}
}

	
