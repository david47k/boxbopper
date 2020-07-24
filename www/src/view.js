import { Vector, Game, Level, Obj } from "wasm-game";

const getRange = length => [...Array(length).keys()];

export class View {
	constructor(gwidth, gheight) {
		this.gameWidth = gwidth;
		this.gameHeight = gheight;
		this.container = document.getElementById('container');
		this.scoreboard = document.getElementById('scoreboard');
		
		this.srcBlockSize = 64; // a default
		this.srcImage = new Image();   // Create new img element
		this.srcImage.loading = "eager";
		this.srcImage.addEventListener('load', function() {
			console.log("image loaded"); 
			document.gameManager.view.setUp(document.gameManager.game.get_level_width(), document.gameManager.game.get_level_height());
		});
		
		window.addEventListener('resize', () => {
			this.setUp(document.gameManager.game.get_level_width(), document.gameManager.game.get_level_height());
		});
		this.setUp(gwidth, gheight);
	}

	setUp(gameWidth, gameHeight) {
		console.log('setting up ...')

		const [child] = this.container.children;
		if(child) {
			this.container.removeChild(child);
		}

		const { width, height } = this.container.getBoundingClientRect();
		this.unitOnScreen = Math.floor(Math.min( width / gameWidth,	height / gameHeight ));
		this.unitOnScreen = ( Math.floor(this.unitOnScreen / 4) * 4 );	// canvas drawImage is crappy, reduce aliasing artifacts
		if(this.unitOnScreen > 256) this.unitOnScreen = 256; // reducing aliasing artifacts - can also split src into individual sprites
		console.log("screen unit:", this.unitOnScreen)

/*		*** not yet supported by browsers other than chrome
		this.srcImageBitmapPromise = createImageBitmap(this.srcImage, 0, 0, (4*this.srcBlockSize), (8*this.srcBlockSize), { resizeWidth: (4*this.unitOnScreen), resizeHeight: (8*this.unitOnScreen), resizeQuality: "high" })
		.then( function(ib) {
			document.gameManager.view.srcImageBitmap = ib;
		}, function() {
			console.log("Failed to create ImageBitmap");
		}); 
*/

		// Because ImageBitmap options aren't yet widely supported, and OffscreenCanvas isn't widely supported,
		// we are using pre-sized images. we can scale down and it looks OK, but we can't scale up.
		// blocksizes 64, 128, 192, 256
		// we return because this method will get called again once the image is loaded
		if(this.unitOnScreen <= 64 && this.srcBlockSize != 64) {
			this.srcBlockSize = 64;
			this.srcImage.src = 'bitmap_64.png';
			return;
		} else if(this.unitOnScreen > 64 && this.unitOnScreen <= 128 && this.srcBlockSize != 128) {
			this.srcBlockSize = 128;
			this.srcImage.src = 'bitmap_128.png';
			return;
		} else if(this.unitOnScreen > 128 && this.unitOnScreen <= 192 && this.srcBlockSize != 192) {
			this.srcBlockSize = 192;
			this.srcImage.src = 'bitmap_192.png';
			return;
		} else if(this.unitOnScreen > 192 && this.unitOnScreen <= 256 && this.srcBlockSize != 256) {
			this.srcBlockSize = 256;
			this.srcImage.src = 'bitmap_256.png';
			return;
		}

		this.scaleToScreen = distance => distance * this.unitOnScreen;
		//this.projectPosition = position => position.scale_by(this.unitOnScreen);

	
		const canvas = document.createElement('canvas');
		this.container.appendChild(canvas);
		this.context = canvas.getContext('2d');
		canvas.setAttribute('width',this.scaleToScreen(gameWidth));
		canvas.setAttribute('height',this.scaleToScreen(gameHeight));
	}
	
	renderImg(dx, dy, sx, sy) {
		sx = sx * this.srcBlockSize;
		sy = sy * this.srcBlockSize;
		if(this.srcImage.complete && this.context) {
			this.context.drawImage(this.srcImage,sx,sy,this.srcBlockSize,this.srcBlockSize,this.scaleToScreen(dx),this.scaleToScreen(dy),this.unitOnScreen,this.unitOnScreen);
		}
	}

	render(game) {
		// render level
		const levelData = game.get_level_data();
		getRange(game.get_level_height()).forEach( function(y) {
			getRange(game.get_level_width()).forEach( function(x) {
				var obj = levelData[y * game.get_level_width() + x];
				if(obj == Obj.Wall) { 
					this.renderImg(x,y,0,0);
				} else if(obj == Obj.Space) {
					this.renderImg(x,y,0,1);
				} else if(obj == Obj.Boulder) {
					this.renderImg(x,y,0,1);
					this.renderImg(x,y,0,2);
				} else if(obj == Obj.Hole) {
					this.renderImg(x,y,0,1);
					this.renderImg(x,y,0,3);
				} else if(obj == Obj.Human) { 
					this.renderImg(x,y,0,1);
				} else if(obj == Obj.HumanInHole) {
					this.renderImg(x,y,0,1);
					this.renderImg(x,y,0,3);
				} else if(obj == Obj.BoulderInHole) {
					this.renderImg(x,y,0,1);
					this.renderImg(x,y,0,3);
					this.renderImg(x,y,0,2);
				}
			}, this);
		}, this);
		
		// render human
		var trans = game.get_transitions_js();
		if(trans && trans.length > 0) {
			trans.forEach( function(ti) {
				if(ti.obj==Obj.Human) {
					document.gameManager.view.renderImg(ti.x,ti.y,0,4);
				}
			});
		} 
		else {
			// why this not work!
			this.renderImg(game.human_pos[0], game.human_pos[1], 0, 4);
		}

		// render scoreboard
		var gm = document.gameManager;
		if(gm.game.have_win_condition()) {
			var mt = "";
			gm.game.get_move_history().forEach( function(c,i) {
				if(i%5==0) { mt += ' ' }
				if(c==0) { mt += 'U' }
				else if(c==1) { mt += 'R' }
				else if(c==2) { mt += 'D' }
				else if(c==3) { mt += 'L' }
			});
			document.getElementById("moves_taken").innerHTML = mt;
			document.getElementById("solved").innerHTML = "You solved the puzzle!";
		}
		document.getElementById("level_num").innerHTML = gm.levelNumber;
		document.getElementById("level_title").innerHTML = gm.levelTitle;
		document.getElementById("best_score").innerHTML = gm.bestScore;
		document.getElementById("num_moves").innerHTML = gm.game.num_moves;
	}
}
