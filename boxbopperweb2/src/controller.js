export class Controller {
	constructor() {
		window.addEventListener('keydown', (ev) => {
			//ev.key takes into consideration state modifiers, locale, and layout. ev.code does not.
			if(ev.key == '`') { 									// ` for Restart
				document.gameManager.restart(document.gameManager.levelNumber);
			} else if(ev.key == 'N' || ev.key == 'n') {
				document.gameManager.nextLevel();
			} else if(ev.key == 'P' || ev.key == 'p') {
				document.gameManager.prevLevel();
			} else if((ev.key == ' ' || ev.key == 'Enter') && document.gameManager.game.have_win_condition()) {
				document.gameManager.nextLevel();
			} else if(ev.key == 'ArrowUp' | ev.code == 'KeyW') {		
				document.gameManager.game.append_move_js(0);				// Up=0, Right=1, Down=2, Left=3
			} else if(ev.key == 'ArrowRight' | ev.code == 'KeyD') {		
				document.gameManager.game.append_move_js(1);				// Up=0, Right=1, Down=2, Left=3
			} else if(ev.key == 'ArrowDown' | ev.code == 'KeyS') {		
				document.gameManager.game.append_move_js(2);				// Up=0, Right=1, Down=2, Left=3
			} else if(ev.key == 'ArrowLeft' | ev.code == 'KeyA') {		
				document.gameManager.game.append_move_js(3);				// Up=0, Right=1, Down=2, Left=3
			}
		});
	}
}
