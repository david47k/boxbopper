// Box Bopper: Sokoban-like game
// Copyright David Atkinson 2020-2021
//
// builtins.rs: This file contains levels that will be built in to the game

pub const BUILTIN_LEVELS: [&str; 79] = [

r"##########
#&   *  O#
##########

title: easy-peasy-6
num: 0
depth: 3
moves: 6
path: RRRRRR
time: 0.0
",

r"##########
#&      O#
#  * *  O#
#   **  O#
#       O#
##########

title: basics-34
num: 1
depth: 19
moves: 34
path: RRRRRRLLLDRRRLLLDRRRLLLDRRR
time: 0.7
",

r"########
###O####
### ####
###* *O#
#O *&###
####*###
####O###
########

title: ten-bucks-10
num: 2
depth: 6
moves: 10
path: DULLRUUDRR
time: 0.0
",

r"########
###OO###
### O###
##  *O##
## *  ##
#  #** #
#  &   #
########

title: quad-bike-56
num: 3
depth: 16
moves: 56
path: LUURRUUDDLLDDRRUUDDRRULDLLLUURRURDLLLDDRRUURULDDDRUULLUU
time: 0.0
",

r"########
##    ##
# * ** #
#OOOOOO#
# ** * #
##  & ##
########

title: hexical-31
num: 4
depth: 10
moves: 31
path: RUDLLLURURRDDLULULRUURRDLULDULD
time: 0.0
",

r"######
# *O #
# ** #
#O @%#
#    #
######

title: yegupumubi-60
num: 5
depth: 11
time: 0.0
moves: 60
path: LRUULDRDDLLLUURRLLDDRUDRRULUURDLDDLLUUURLDDDRRUULRDDLLURUULD
seed: 654
width: 6
height: 6
wall_density: 20
num_boxxes: 4
",

r"######
#   ##
# * %#
# *#*#
#O  O#
######

title: zizikamabe-26
num: 6
moves: 26
depth: 7
path: DULULLDRRLLDDRULUURRDLULDD
time: 0.0
seed: 12
width: 4
height: 4
wall_density: 20
num_boxxes: 3
",

r"######
#  &##
# *@O#
# # *#
# O  #
######

title: pavahibete-30
depth: 5
moves: 30
path: LLDDDRRRUDLLLUUURRDDRDLUUULLDR
time: 0.0
num: 7
seed: 896
width: 4
height: 4
wall_density: 20
num_boxxes: 3
",

r"######
# O  #
# *  #
# #@ #
# *&O#
######

title: kihuwolobi-32
num: 8
depth: 7
moves: 32
path: URUULLLDDDRRUUDDLLUURLDDRRURUULD
time: 0.0
seed: 997
width: 4
height: 4
wall_density: 20
num_boxxes: 3
",

r"######
#&@  #
# ** #
#O#  #
#  O #
######

title: boweponeri-42
num: 9
moves: 42
path: DDDRRURUULDRDDLLLUUURLDDDRRRUULDULULDRRRUL
seed: 718
width: 4
height: 4
wall_density: 20
num_boxxes: 3
depth: 6
moves: 42
time: 0.0
",

r"#########
#     # #
# @&*#  #
#O   #  #
# #   @ #
#   #  ##
#   #   #
#########

title: nuyolonoyi-38
num: 10
moves: 38
path: DRDRDRURUULDRDLLLULUURDDLLLDDRRURRLULL
seed: 43
width: 7
height: 6
wall_density: 20
num_boxxes: 3
depth: 12
time: 0.0
",

r"##########
## #&#   #
## O* *# #
#  #     #
#  #  #  #
##   * # #
# @#*   ##
#O OO # ##
# #    # #
##########

title: zegasaheke-53
num: 11
moves: 53
path: DRDDLDDRDLLLRRUUUURRRRUULLDURRDDLLLULDDDDRDLURRULULDD
path: DRDDDLDURRDLDLLLRRRUULDRDLUUUURRRRUULLDURRDDLLLULDDDD
seed: 66
width: 8
height: 8
wall_density: 23
num_boxxes: 5
depth: 18
old_time: 1.0
time: 1.3
",

r"##########
###    ###
### *    #
### * ## #
#OOO *   #
#OOO*#* ##
#### # * #
####  &  #
##########

title: getting-tricky-101
num: 12
moves: 101
depth: 32
path: LLUULUURLDDRULUUURRDLULDDDUURRRRRDDLLLLLLRRDLLRRDDRRUURULLLLUURRDLULDDRDLRDDRRRUULULLULDUURRRRRDDLLLL      
old_time: 2.10
time: 3.5
",

r"##########
###   #O #
###  *OOO#
##  * #@O#
## ##*# ##
#   *  * #
#   #    #
#######& #
##########

title: too-hard-yet-97
num: 13
moves: 97
path: URULUURUULDLLULLDRRRRURDLDDDRDDLULLUUUDDDRRUUUULLULLDRLDRURRRDDDDLLURDRUUUDDLLUULURDLLLDDRRRRDRUU
depth: 36
time: 0.7
",

r"###########
#### OOO###
####OOOO###
#  ###* ###
# * *  ** #
#& * *    #
#   ###   #
###########

title: lucky-seven-105
num: 14
moves: 105
path: UURDRRDRRUUURULDDDDLLURDRUURULLRDDLLLLLDDRULURRRRDRUUUDDLLLLDDRULURRRDRUUDDRUDDRRUULLUULRDDRRDLDLUUUDDLUU
depth: 39
time: 3.95
",

r"##########
#   ##  ##
# # * * ##
#  @O#  ##
## #O&O###
##*###@###
#        #
#   ## # #
######   #
##########

title: need-solution-stat-162
num: 15
moves: 162
path: LULUULLDDRDDUULUURRDRDDRRURUULDLLDDRRURULDDDDRRDDLLUULLLDLLURUUURLLUURRDULLDDRDDDRRRRDDRRUULLUUURUULDLLDURRDDDDRRDDLLUUUDLLLDLLURRRRRDDRRUULRDDLLUULLLLUUURLLUURRD
depth: 48
time: 0.09
",

r"#########
#  @O # #
#  * OO##
#  * # ##
# #* % ##
#  *@*  #
#    O ##
# ### ###
#########

title: widorehadi-44
num: 16
moves: 44
depth: 18
path: LDRDRUUUDDLLLDRLLLUUURRLUURLDRRDLLLDDRRURLUU
old_time: 6.82
time: 10.5
seed: 11249
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
# O    ##
# ##** ##
#O@  *###
#O    ###
#O*## ###
# &O#@* #
#       #
#########

title: lozomulehu-118
num: 17
depth: 33
moves: 118
path: LUUUUURRRDURRDLULLLLDDDRRULRDRRULRURULLDDRDLLLURDRRUULDLDLDLDDRRULRDRRUUULLULDDURRRULUURRDLULLRDRDLLRRDDDDRRULDLLLLLUR
old_time: 1.71
time: 2.1
seed: 11860
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
##    OO#
#####*  #
#O##    #
#&*     #
#  @*@  #
# #  *#O#
# ##O * #
#########

title: juzehiguju-67
num: 18
depth: 33
moves: 67
path: RRRDRURRDDDLLUULUURRDLLLLDRDRLULLURRRRRRDDDLLUULURDRURDUUULULDDDRUU
old_time: 14.74
time: 20.8
seed: 11230
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
## ##   #
# O    O#
#  *@*# #
## O  *O#
#O## #  #
#**  # *#
#   O##&#
#########

title: fewuhopiga-82
num: 19
depth: 36
moves: 82
path: UUULLLULURRDLDDDDLLLURRDRUUUURUURRDDDLLUURLDLDDDUUULULLDRRURRURRDLLLLRRDDRRUDLDDRU
time: 1.07
seed: 11452
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"######
#    #
#@ * #
#%@ @#
#@  @#
######

title: kefikuboju-26
num: 20
moves: 26
path: RUULDURRRDLLDDRULUURRDLULD
seed: 209
width: 4
height: 4
wall_density: 20
num_boxxes: 6
depth: 7
time: 0.0
",

r"######
#O *&#
#@@* #
## **#
#OO O#
######

title: jodihitudu-26
num: 21
moves: 26
path: LLDDUURRDDLDLRURUULLDDRRUL
depth: 9
time: 0.0
seed: 914
width: 4
height: 4
wall_density: 20
num_boxxes: 6
",

r"######
#&@  #
# ** #
# #  #
#O  O#
######

title: hanemuwazi-33
num: 22
moves: 33
path: DDDRRURUULDRDDLLLUURRDRDLLRUUURDD
depth: 8
time: 0.0
seed: 235
width: 6
height: 6
wall_density: 20
num_boxxes: 3
",

r"#########
# #    O#
#    *&##
# #  #  #
#O**#O  #
#   #   #
#########

title: gagekopevo-42
num: 23
depth: 12
moves: 42
path: ULLDLLLDDDRRUUDDLLUUURRDDLRURURLLURRRDDRDL
seed: 0
time: 0.0
width: 7
height: 5
wall_density: 20
box_density: 5
num_boxxes: 3
",

r"#########
#&# *O O#
#**    ##
# # O#* #
#O* #@  #
#O  #O* #
#########

title: gagekopevo-ii-38
num: 24
depth: 16
moves: 38
path: DDDUURRURDRRULRDDRDDLUUULURDLLLURDLDDL
seed: 0
time: 0.0
width: 7
height: 5
wall_density: 20
box_density: 20
num_boxxes: 7
",

r"#######
# @   #
# *O**#
#&@  O#
#  # O#
#######

title: vijarozajo-56
num: 25
depth: 17
moves: 56
path: RRUURRDDUULLDDRLUURRDLDLLLUURDURDRRULLDLLDDRULURRURRDLDL
time: 0.0
seed: 58
width: 5
height: 4
wall_density: 20
num_boxxes: 5",

r"#########
## O#&###
#O  #@  #
#O  * # #
# #   @ #
# * ##* #
#  #O * #
#########

title: ruzinerihu-57
num: 26
depth: 20
moves: 57
path: DRRDDLLULLDDLDLUUURLDDRRUURRDLULLLDDRRURULLRUDRRDRRDDLUDL
time: 0.0
seed: 13056
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#########
#   O   #
#O  @ # #
###*@#  #
##@&* # #
# # #   #
# # * O##
#########

title: semamogepo-73
num: 27
depth: 14
moves: 73
path: DDRRURRUUUULLDLLLURRRRRDDDDLLDLLUUUDDDRRUULRDRRUUUULLLLLDRDDRRDDLLUUUULUR
time: 0.0
seed: 13028
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#########
## * O  #
#&*O*O# #
# # #   #
# * @# ##
##  *  ##
##O#  O #
#########

title: zarofeveki-81
num: 28
depth: 19
moves: 81
path: RURRRDDRDDLLLRRRUULUULLDDDUULLDDRDRRRLLULLUURURRRDDRRUULRDDLDDLLLULLUURRRLLLDDRRU
time: 0.0
seed: 13031
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#######
# @ OO#
#O* *##
# ** O#
# O&* #
#######

title: wohurojaga-59
num: 29
depth: 15
moves: 59
time: 0.0
path: LLUUURRRLLLDDDRRURUDLDLLUUURRDRDRDLUULULLDRLDDRUUDRRDLURULL
seed: 171
width: 5
height: 4
wall_density: 20
num_boxxes: 6
",

r"#########
#    ## #
# # **  #
#O%#   ##
# ** @ O#
#  *#   #
##    OO#
#########

title: dijumodibi-75
num: 30
depth: 31
moves: 75
path: DRDULLDRDRRRRULULLLULUURRRDDUULLLDDRDRRRRUULDLDLLULUURRRDDRDLLDLLUDRUDDRRRU
time: 5.05
seed: 13013
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#######
## %# #
# *@@ #
##    #
#    *#
# O* O#
#######

title: hizozuwozo-64
num: 31
depth: 20
moves: 64
path: LDURDDRRULDLURDDDRUDLLULLDRRURUULLDRDDLLURDRURRDLLULUURRRDDLUDLU
time: 0.0
seed: 338
width: 5
height: 5
wall_density: 25
num_boxxes: 5
",

r"#########
# # # #O#
#OO*    #
## ## **#
#O  #   #
#  * #*##
##  O @&#
#########

title: yimosomodu-88
num: 32
depth: 31
moves: 88
path: LLLULULUURRRDDRRULRULLLLLDDRDRDRRUUDDLLULULLDRDRRULDLURULUURRRRRDDLDDLRUURUULDDDULURULLL
time: 0.0
seed: 13007
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#########
## # ## #
# #O ####
#  @#   #
#O  #** #
#**#%*  #
#O   O  #
#########

title: hegovitafu-49
num: 33
depth: 15
moves: 49
path: DLLUURUDLLURDDDLURDRRRRURUULDDRDLLLLLRRRRULRUULDD
time: 0.0
seed: 13002
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"#######
# #   #
#&*  @#
# O@  #
##*#@ #
#O    #
#######

title: humoseyiju-80
num: 34
depth: 21
moves: 80
path: RRURRDLDDRDLUUURDLDDLLUURLDDRRUURUULDDLLDDRRRUULUULDRDRDDLLLUULURDDDRRRUULLRUULD
time: 0.0
seed: 484
width: 5
height: 5
wall_density: 25
num_boxxes: 5
",

r"#########
#   #   #
#O*@    #
#   * @&#
## *# O #
## O *  #
#   ###O#
#########

title: komujodujo-60
num: 35
depth: 24
moves: 60
path: ULDRDDLLLLULULUURDRDRRRLLLULLDRRRRURRDDDLLLLDLUUURULRDDURRUL
time: 3.68
seed: 13036
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"########
#  O  ##
#O #O  #
#   *# #
#O**&* #
# #   ##
########

title: ribubobono-70
num: 36
time: 0.0
moves: 70
depth: 17
path: ULLUURRRDRDDLDLLUDRRULULRDDLURRRUULULLLDLDDRUDRDRRULULLULURDDDLURRRDLL
seed: 3414
width: 6
height: 5
wall_density: 35
num_boxxes: 4
",

r"########
#    @ #
# O    #
##O  ###
##** *&#
# O#   #
########

title: kesokehofu-69
num: 37
time: 0.0
moves: 69
depth: 24
path: LDLUULUURDRRULLLDDLDRRULUURRDLULDRDDDRRULDLUULUURRDLULDLLURRRDDDLURUL
seed: 3836
width: 6
height: 5
wall_density: 35
num_boxxes: 4
",

r"############
# ##   # *O#
#OO* # * ###
## *##O*  O#
# O  *&## ##
############

title: zupupomiwi-60
num: 38
time: 0.0
depth: 13
moves: 60
path: UUULLDLLRRURRDDDLLLUDRRRUUULLDLRURRDDRRRLUURLDLDLDLLLUURURRD
seed: 14138
width: 10
height: 4
wall_density: 33
box_density: 15
num_boxxes: 6
",

r"#######
#OO   #
# #&*O#
# @ *##
# *O* #
# *  O#
#######

title: nuvoradene-69
num: 39
time: 0.1
depth: 23
moves: 69
path: RLDDDRRULULUURDULLLDDDDRRULDLUUUDDRRRRDLLUUURDLDDRRULDLULLDRRRUUUULDD
seed: 282
width: 5
height: 5
wall_density: 25
num_boxxes: 6
",

r"#######
#   OO#
##*   #
# * *O#
# #** #
#%*O O#
#######

title: fanugudiku-64
num: 40
depth: 25
moves: 64
time: 0.0
path: RRUURLDDRURUULDRDLDLLLUURRRUULLDURRDDDDLLLUURRDRDLLRUULUURDDDRUU
seed: 413
width: 5
height: 5
wall_density: 25
num_boxxes: 6
",

r"########
## # O #
# * *O #
#O *&*@#
# #@#  #
#O     #
########

title: hivigekuru-81
num: 41
time: 0.1
depth: 25
moves: 81
path: LURURRDDLDDLLLLUUURRLLDDDRRUULULDRRDDRRUULRRUULLDLDLLDUURDRRUURRDDLDDRULUUDLLLURR
seed: 5543
width: 6
height: 5
wall_density: 35
box_density: 20
num_boxxes: 6
",

r"##########
#O O #   #
#   ** * #
#  O O *&#
##########

title: fabozajoyu-48
num: 42
depth: 22
moves: 48
path: LLLLLUURDLDRRRULLDLULLDRURRRRURRDLLRRDLLULLLLDLU
time: 0.0
seed: 465
width: 8
height: 3
wall_density: 20
num_boxxes: 4
",

r"##########
# %*    O#
#  ***#  #
####   OO#
##########

title: bojudogusa-42
num: 43
depth: 23
moves: 42
time: 0.0
path: RRRRRDDLLLULURRDULLLLDRRRDRRRUULLLLRRDLDRR
seed: 209
width: 10
height: 5
wall_density: 20
num_boxxes: 4
",

r"##########
#OO   O*O#
#O ****# #
#     O*&#
##########

title: pazukopubi-45
num: 44
time: 0.1
depth: 23
moves: 45
path: UULLLLLLRRRRRRDDLLUDLLURURRLLDDRULLLLDRRRUULL
seed: 137
width: 8
height: 3
wall_density: 20
num_boxxes: 6
",

r"##########
#O%*O   O#
# O***  *#
# *     O#
##########

title: nuzusumoya-45
num: 45
time: 0.5
depth: 25
moves: 45
path: RRRLLLDLDRRRULRRRDRRULLLULLLRDDRRRRUULLLLDDRU
seed: 73
width: 8
height: 3
wall_density: 20
num_boxxes: 6
",

r"########
#O&*  ##
# **#  #
# *    #
###*  O#
#    O #
#  # OO#
########

title: zizejitapo-70
num: 46
time: 0.4
depth: 28
moves: 70
path: LDDRRRRUULLLDLDRRRLLUURDLDRUURRDDURDDDLLLUURRDDUUUULLDLDRRLUURRDDURDLD
time: 0.4
seed: 10368
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"############
# *     O  #
# **@*     #
#&*  #   O #
####OO O#  #
############

title: viwaheyejo-87
num: 47
depth: 33
moves: 87
path: UURRRRRLLDRRDDLLUDRRUUURDULLLLLLDDRRLLUURRDRRRDDLLULUULLDDRRURRLDLLLURRURRRDDRDLUUURDDR
time: 8.4
seed: 14082
width: 10
height: 4
wall_density: 33
box_density: 15
num_boxxes: 6
",

r"########
#O   ###
#   ** #
# @*  O#
# &#* O#
#  # ###
#O   # #
########

title: zajebehifu-49
num: 48
depth: 25
moves: 49
path: URRDRRULLLUURDLDRLLULDDDRDRRUURRUULDULLULDRDRRLDR
time: 0.9
seed: 10502
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
#&*O   *   O #
##** # #   * #
#O O##OO   * #
#         # ##
##############

title: kewikidesi-111
num: 49
depth: 30
moves: 111
path: RDURRDLDDRRRUUURRDDDLLLLLUUURRRDDDLLLLLURDRRRRUUULLLLDURRRRDDDLLLULUURDURRRDDRRURRURRDLRDLLLLLRUURRDRDLLLUULLLL
time: 2.3
seed: 18949
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"#########
# O# ## #
#       #
#& O#**##
#* #  * #
#OO@  @ #
#  #    #
#########

title: famuzuhize-180
num: 50
depth: 40
moves: 180
path: RDDLUURURRRDDLDLRURUULLDLDLDDRURRDRUUDDRRULRULDDLLULLULURUULDRRRRRDDRDDLLLUURURULLLRRDDLDLLUUUDDDRRDRUDRRULUDLLLRURUULLLDDLDDRUUDRRURUULLLLDDRDRRRRRULDLLLLULUURDDUURRRDDRDLLURUURDD
time: 0.5
seed: 13030
width: 7
height: 6
wall_density: 25
box_density: 15
num_boxxes: 6
",

r"##############
#%@  O# #   ##
# # *        #
####*O*## **##
#  #     #OO##
##############

title: yidevuyale-75
num: 51
depth: 38
moves: 75
path: RRDRRULLLRRRDDDLUURRRRRURRDDULLLLLLDDRULUULLDRURDRRRRURRDLDULLLLLRDDLLUULUR
time: 0.0
seed: 18081
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
#     O#
# * #O #
##**   #
##  @*##
# ##%  #
#O     #
########

title: ragamosufa-59
num: 52
depth: 24
moves: 59
path: RUURUULLLLLDRURRRDDDULDLLURRDRULLLULURRRRLLDDLDRURRDDRDLLLL
time: 0.3
seed: 10877
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"############
#  #       #
#OO%# #  O #
# ****** # #
##O     O  #
############

title: zoruwiruru-116
num: 53
depth: 40
moves: 116
path: DULLDRDRRRRRUDLLURDLLLLUURDRDRRULLLULLDRDRRULDLURRRRDRRULLLLLULRDDLURRRRRRUULLLDURRDRDDLLULLRRDLLLURRRRDRULURURRDDDL
time: 0.9
seed: 14324
width: 10
height: 4
wall_density: 33
box_density: 15
num_boxxes: 6
",

r"########
#O O*&##
##** # #
#  * ###
# #* O #
#O O # #
##    ##
########

title: nijefigiwe-83
num: 54
depth: 20
moves: 83
path: LDDLDRDDLLURDRULURUUULDDDRDDLLULUURLDDRDRRUUUUULLDRRDDLUURDDDDLLULUURURDRDDLLRRUULD
time: 0.0
seed: 10623
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
#OO%   #     #
#  *  * O*  ##
#    #**  # ##
# ##O*O    ###
##############

title: yutehezena-101
num: 55
depth: 35
moves: 101
path: RRDRLLDDRRURRURLDDRULLLDRLLLULURURRDDRDLLRURRRDLLURULLLLLLRRRRRRURDURRDLLLDDRULULRURRDLDLLULLLLDLUDLU
time: 11.6
seed: 18175
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
##&##O##
##*O  O#
#  ** O#
#  * O #
##  *  #
#  #   #
########

title: difulivewa-46
num: 56
depth: 22
moves: 46
path: DRRDULLDRRLDRRDDLURRUUDLULLLLDRDRRDRULUDLUURRU
time: 0.5
seed: 10343
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
#O  #  #  # ##
#      @   @&#
# * ##* **O ##
#  ##  O   O #
##############

title: topiyogayi-66
num: 57
depth: 36
moves: 66
path: LDDLLLURURDULULDLLLLLDLDLUURRRRRRDDLURRRUULDDRDRLLUULLLLLDLLURRRRR
time: 5.3
seed: 18370
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
#     ##
# #  ###
# #*O ##
## OO*&#
#@  ** #
#   # O#
########

title: yofajumeha-77
num: 58
depth: 24
moves: 77
path: LULULDDLDDRUUURRDLRRDDLURULULLDLDDRURRURDULULLDRLLDDRULURURUULDDDLDDRURUDRRUL
time: 0.0
seed: 10582
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
# O O   #  # #
#*# #  # *   #
#O###  * @@  #
##&*       O #
##############

title: dufesuzadu-96
num: 59
depth: 36
moves: 96
path: RRRUUULLLLDURRRRDDDRRRUURRLLDLLDLUURULLLRRRDDRRDRRURRDLLULLURURDLDRDRRULRULLULDLDDRRLLULLDLUURUL
time: 1.9
seed: 18204
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
#  # O #
#&*   ##
##* #  #
## @ O #
#  * @ #
#   # O#
########

title: namilekafo-56
num: 60
depth: 26
moves: 56
path: RRRURDDRDLRDDLUUDLLDLLURRURRURDDUULULLDULDDRRLDDLLURDRUR
time: 0.1
seed: 10238
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
#    O     *&#
#  O# *####*##
#    @* #   O#
# O *   O    #
##############

title: tezamugeho-66
num: 61
depth: 34
moves: 66
path: LDULLLLLLLLDDDRRUDRRULLLRRRDRRURRDLLURRUULLLLLDULDRDRDLLLLULULURRR
time: 16.1
seed: 18274
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
##O  ###
# *O *%#
# # ** #
# O* ###
#     ##
# O #  #
########

title: bizosaloya-69
num: 62
depth: 25
moves: 69
path: LLDLURRRDLULULDDDLLUURRRRDLULLLDDRDDRURULRURULLDDRDLDLLUUUURLDDRDRRUL
time: 0.0
seed: 10608
width: 6
height: 6
wall_density: 35
box_density: 15
num_boxxes: 5
",

r"##############
##    # O  O #
# ** *#** ####
#   O      @%#
# #O# #  #  ##
##############

title: tebehipiza-83
num: 63
depth: 43
moves: 83
path: LLLLLLLLUURDLDRRRRRRRLLUULLDRDLLLUULLLDRDRRRRRRRLULURRLDDLLLLLLULLDRRRRUULDRDRRDRUU
time: 0.7
seed: 18518
width: 12
height: 4
wall_density: 33
box_density: 13
num_boxxes: 6
",

r"########
#&@  O #
#O *  ##
##O#** #
#      #
## @O* #
########

title: pirayikena-77
num: 64
depth: 27
moves: 77
path: DRRURRDDRDLUUULLDLLURDRRURDDRDDLULUDRRULULLLURDLDDRRURULLULDRRRULLRRDDRDDLUUU
time: 0.0
seed: 5916
width: 6
height: 5
wall_density: 35
box_density: 20
num_boxxes: 6
",

r"#########
#  OO   #
#  *##  #
#@   * ##
## *#  ##
##O  #  #
# #**O@##
# #   % #
#########

title: bevanehova-156
num: 65
depth: 49
moves: 156
path: LLLURULRDDRRUUUULLLDDUURRRURULLLLDLLURRDDRRRDDDDLLUULULURUULLDRDRDDRDDRRUUUUUULLLDDRRDRDDDLLUULULURUURRRDDDDDUUUUULLLDDRRLLUURRRDDDDUULLLULLURRDDDDRDDLUUUUU
time: 0.3
seed: 11589
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
#   ###O#
#   *   #
##*#%** #
#  #* O##
#O@# ####
#  * O  #
#O    ###
#########

title: falodalore-142
num: 66
depth: 45
moves: 142
path: DRRUDLLURURDULLLULLDRRRRDDLDDLDLLUUURUDLDDDRRURRDLLURUUUULULLDRDDLDDRRDRULLLDRULUURDLDRRRUURUULLULDDDDUUURRRDRRULLLLULDDDUURRRDDLDDLLDLURRRDLL
time: 1.0
seed: 11208
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
#&*OO * #
##*O* OO#
## #* # #
#OO  *  #
## #*#  #
#    #  #
### #   #
#########

title: rotujeleti-122
num: 67
depth: 35
moves: 122
path: RDRRURDRRDDLLLUDRRRUULLDLDRRLLLLDDRRUDLLUURRURUULDRRRULLDLLLDDRRUDLLUURRRURRDDDLDDRUUUDLLLDDLLUURLDDRRUURUULLLDURRRDDLURUL
time: 0.8
seed: 11415
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"#########
#    * O#
#  *O# *#
# * ## O#
##*     #
## #   ##
#O *  @%#
# # O  ##
#########

title: wosusenove-72
num: 68
depth: 35
moves: 72
path: LUUUUULLLDDULLDRDRRRLLLUURDUURRRRDLDDDLDRDLULLLUURRRLLLULURRLLURRRRRDDDD
time: 9.2
seed: 11543
width: 7
height: 7
wall_density: 35
box_density: 15
num_boxxes: 7
",

r"########
#  OO O#
##  # ##
# * O###
#&* * O#
## * * #
# ##   #
########

title: keyapegika-80
num: 69
depth: 32
moves: 80
path: RURRDRRDDLLURLULULLDRRUUULDDLDRDRUUULURRRLLDDDDRDRRULULRDDLULUUULURDDDDRRULDLUUU
time: 0.0
seed: 12733
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
#O# O  #
#  *&* #
#O#*# ##
# *  @ #
#  O  ##
# # #  #
########

title: pokaviluwa-93
num: 70
depth: 31
moves: 93
path: LURRRDLULLDDDRDRULLLDRURRUULULDDURRDDLLDLLUUURRDDLRRRUULULDDURRDDLLDLLURRUURRRULDDDDLLURLLLDR
time: 0.0
seed: 12806
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
# OO&  #
#  #@*##
# #  *O#
#    # #
#   ** #
#O#    #
########

title: wutovokeha-116
num: 71
depth: 27
moves: 116
path: LLDLDDRRDDRUDRRULLLUURDLDRULLLUUURRRDDDLDDRRRUUULRDDDLLLULULUUURRRRDULLLDLDDDURRRUURULLRDDDLDDRUUUUDDDDRRULDLUUURUUL
time: 0.2
seed: 12935
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
#  # O #
# *#*&##
#  O*@##
# *#OO #
#     ##
#    ###
########

title: digufamose-94
num: 72
depth: 26
moves: 94
path: DLDDLLLUURDLDRUULUURDDDLDDRRRUUULRRUULDRDLDDRUUUDDDLDLLLUURURRURDLLLLUURDLDRRRDDDLLURDRULLULUR
time: 0.0
seed: 12120
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
#      #
# O*@@O#
#   #  #
# O #* #
# #*  ##
#  &   #
########

title: muyahuzabo-98
num: 73
depth: 22
moves: 98
path: LLUUUUURRDURRRDDLDURUULDLRRDDLURUULLDRLLULLDDRDRULLUURDRRLLDLDDDRRUUULUURRDLURRRDLULLDDDDRDRUUUDRU
time: 0.0
seed: 12859
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
## #   #
# * * ##
# O ** #
#  %*  #
## O # #
#O    O#
########

title: dukosudeja-59
num: 74
depth: 24
moves: 59
path: DRDRRUULLLUURDDDURRULLUURDDRDDDLLLLUULUURRDDRRUUULDDRRDDULL
time: 0.8
seed: 12975
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
# ##&  #
#   * ##
##*  # #
## ** O#
#O OO *#
#  # O #
########

title: mujalatuki-67
num: 75
depth: 21
moves: 67
path: DLDDLDRUUULDDRDRDRRULULRDDLULUDRRULDLLUUURRDDUULLDDRDRLULUURDDURDRD
time: 0.0
seed: 12349
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
## #   #
#%*    #
#OO** O#
# #  * #
##  * ##
#   #O #
########

title: nomezogefi-35
num: 76
depth: 19
moves: 35
path: RRRRLDLLRDDRUURRDLDLURULLRRUURDLLLL
time: 0.0
seed: 12436
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"########
###   O#
# **&  #
#   * O#
#O#O   #
#O #** #
#    ###
########

title: zijesebaze-68
num: 77
depth: 25
moves: 68
path: ULDDRLLLURDLDDRDRRUUUUDRDRDLUULLLLDDRDRRUURULLLULDDUURRURDURDDLLLULD
time: 0.8
seed: 12212
width: 6
height: 6
wall_density: 25
box_density: 15
num_boxxes: 5
",

r"#################
#               #
# *  *  **  * * #
#  **  * &* * * #
#  *   *  * * * #
# *     **  *** #
#               #
# O O O  O  OO  #
# O O O  O  O O #
# O O O  O  O O #
#  O O   O  O O #
#               #
#################

title: winner
width: 15
height: 11
num_boxxes: 23
",

];

