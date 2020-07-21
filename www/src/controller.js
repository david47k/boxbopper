
import { Move } from "wasm-game";

const MOVEMENT_KEYS = {
	[Move::Up]: [87,38],
	[Move::Right]: [68, 39],
	[Move::Down]: [83,40],
	[Move::Left]: [65,37],
	'reset': [82],
}

export class Controller {
	constructor() {
		window.addEventListener('keydown', ({ which }) => {
			this.movement = Object.keys(MOVEMENT_KEYS).find(key => MOVEMENT_KEYS[key].includes(which));
		});
		window.addEventListener('keyup', ({ which }) => {
			if(this.movement == Object.keys(MOVEMENT_KEYS).find(key => MOVEMENT_KEYS[key].includes(which))) {
				// we can only consider a keyup if it applys to the current keydown that we have
				this.movement = undefined;
			}
		});
	}
}

	
