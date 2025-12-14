# Tarkov State Machine
## Game Maine Menue
* Escape From Tarkov - Character select
* Character - Stash and loadout management
* Traders - Buying and Tasks data
* Hideout - Build Modueles, Craft things
* Exit - exit Game
There are also several "tabs" at the bottom of the screen that can do the same thing as the menu, these usually pop you
back to the last of the majors listed above:
* main menu
* Hideout with notifications icons
* character with notification icons
* traders
* FleaMarket (Player to Player trading, and a good way of searching through items also available from traders)
* Builds (this is the gun crafting section)
* HandBook (lore and basic data)
* Messages (UI POPUP, that you can get messages from Players, and traders .. that can been quest rewards, money from sold flea,
    and insuranace returns)
* Survey (something they rarely uses)
* Cog Icon (settings) 
* There is a PVE or PVP toggle in the bottom right

## Stash Management
All of the following list is considered Stash Management that have MANY sub screens that may or may not be useful to this program
I would consider this all a part of the same time tracking, its a non-trivial part of the game, nicknamed stash tetris.
* Character
* Traders
* Hideout

## Escape from Tarkov
Select Character is the next options
* SCAV
* PMC

## Character Selected (SCAV and PMC)
* Map Selection
* Time Of day
Next States:
* Next, Back, map(?), Ready

## Practice mode (PMC?) Map Selected Selecctde
This lets you make a custom local game if you want to testsoca
Next States:
* Next, Back, Ready

## Map Selected: (PMC) Insurance
Insure with one of 3 vendors that sometimes you'll get your stuff back
Next States:
* Next, Back, Insure, Ready

## Deploying (PMC, SCAV)
The following stages you can still back out of:
* Loading Map
* Caching
* Loading OBjects

You cannot back after this:
* PVE: Starting Local Game
* PVP: I think there is a matching / waiting for players ... i think you can back out of matching (will confirm later)

## Get Ready
Just a simple countdown to load in.


## RAID
This is all gameplay FPS and Menu for gear and loot health etc.  There are several tabs in your inventory but they mostly do not impact the game

## Transfers
this will actually move the state to Deploying again as it as map movement without going through all the initail selections
there will be word hints as to which map we're moving to
This should start a new timer.

## RAID END
This has the status
* Survived
* Died
* MIA

Next States:
* next, main menu

## Raid Ended: Kill List
This will have Location, time code (in specific map), if its a scav or PMC, their "name", and "status" which is how killed, and distance, with what weapon.

Next States:
* Next, Back

## Raid Ended: Statistic
This has a lot of details for the "last map"

Next States:
* Next, Back

## Raid Ended: Experience
what got my experience and a level up progress bar

Next States:
* Next, Back

NEXT takes you back to main menu 
