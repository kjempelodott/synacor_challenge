class Event(object):
    def __init__(self, assertion, toast, *actions, **kwargs):
        self._assert = assertion
        self._actions = actions
        self._toast = toast
        self._unlocks = kwargs.get('unlocks', [])
        self._locks = kwargs.get('locks', [])
        self._once = kwargs.get('once', True)
        self.callback = kwargs.get('callback')
    def __call__(self, player):
        assert(self._assert(player))
        if self.callback:
            yield from self.callback(player, self._actions)
        else:
            yield from self._actions
        print(self._toast)
        for event in self._locks:
            open_events.remove(event)
        for event in self._unlocks:
            open_events.append(event)
        if self._once:
            open_events.remove(self)
            player.save_progress()
    def __repr__(self):
        return 'Event: ' + self._toast

def shuffle_coins(player, actions):
    from itertools import permutations
    player.dry_run = True
    for coins in permutations(actions):
        actions = list(coins)
        for coin in actions:
            yield coin
            if 'released onto the floor' in player.text:
                yield 'take red coin'
                yield 'take blue coin'
                yield 'take shiny coin'
                yield 'take concave coin'
                yield 'take corroded coin'
                break
        else:
            break
    player.actions.extend(actions)
    player.dry_run = False


bookshelf       = Event(lambda x: 'bookshelf' in x.text,
                        'Read book',
                        'look strange book')

tablet          = Event(lambda x: 'tablet' in x.inventory,
                        'Wrote on tablet',
                        'use tablet',
                        unlocks=(bookshelf,))

teleporter      = Event(lambda x: 'teleporter' in x.inventory,
                        'Teleporting ...',
                        'use teleporter',
                        unlocks=(tablet,))

locked_door     = Event(lambda x: 'door is locked' in x.text,
                        'Locked door! Going back ...',
                        'south',
                        once=False)

unlock_door     = Event(lambda x: '_ + _ * _^2 + _^3 - _ = 399' in x.text,
                        'Placed coins in the sockets!',
                        'use red coin',
                        'use blue coin',
                        'use shiny coin',
                        'use concave coin',
                        'use corroded coin',
                        unlocks=(teleporter,),
                        locks=(locked_door,),
                        callback=shuffle_coins)

five_coins      = Event(lambda x: len([c for c in x.inventory if 'coin' in c]) == 5,
                        'Found the five coins!',
                        unlocks=(unlock_door,))

lantern_and_can = Event(lambda x: 'empty lantern' in x.inventory and 'can' in x.inventory,
                        'Lantern lit!',
                        'use can',
                        'use lantern',
                        unlocks=(locked_door, five_coins))


open_events = [lantern_and_can]
