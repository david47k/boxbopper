import * as wasm from "wasm-game";

import { GameManager } from './src/game-manager'

const gameManager = new GameManager();

;(function () {
    function main( tFrame ) {
        gameManager.stopMain = window.requestAnimationFrame( main );
        //update( tFrame ); // Call your update method. In our case, we give it rAF's timestamp.
        //gameManager.update(tFrame);
        gameManager.render();      
    }
    
    main(); // Start the cycle
  })();
