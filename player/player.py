import re, random
from subprocess import Popen, PIPE
from event import open_events

class EatenByGrue(Exception):
    pass

OPPOSITES = { 'north'  : 'south',
              'west'   : 'east',
              'south'  : 'north',
              'east'   : 'west',
              'up'     : 'down',
              'down'   : 'up',
              'ladder' : 'ladder' }

DEBUG = 0
class Player(object):

    RE_EXIT = re.compile('There (are|is) (\d+) exit.+')
    RE_OBJ = re.compile('Things of interest here.+')

    def __init__(self, vm):
        self.dry_run = False
        self.game = None
        self._vm = vm
        self.reset()
        self.inventory = set()
        self.progress = []

    def save_progress(self):
        self.progress = [a for a in self.actions]
        self.saved_exits = [e for e in self.exits]
        if DEBUG == 1:
            print('Progress saved ...', self.progress, self.saved_exits)
        
    def reset(self):
        self.kill()
        self.game = Popen([self._vm], stdin=PIPE, stdout=PIPE)
        self.text = ''
        self.actions = []
        self.last_move = ''

    def getline(self):
        line = self.game.stdout.readline().strip().decode('utf-8')
        self.text += line + '\n'
        if DEBUG == 2:
            print('`', line)
        if line == 'You have been eaten by a grue.':
            raise EatenByGrue
        return line

    def read_exits(self, n):
        exits = [self.getline()[2:]  for i in range(n)]
        if n == 1 or 'maze' in self.text:
            self.exits = exits
            return
        self.exits = list(filter(lambda x: x != OPPOSITES.get(self.last_move), exits))
        
    def read_text(self):
        self.text = ''
        objects = []
        line = self.getline()
        while line != 'What do you do?':
            m = self.RE_EXIT.match(line)
            if m:
                n = int(m.groups()[-1])
                self.read_exits(n)
                line = self.getline()
                continue

            m = self.RE_OBJ.match(line)
            if m:
                line = self.getline()
                while line:
                    objects.append(line[2:])
                    line = self.getline()
            line = self.getline()
        for obj in objects:
            self.inventory.add(obj)
            self.do_action('take ' + obj)
            if DEBUG == 1:
                print(self.inventory)
            self.flush()
            self.save_progress()
    
    def flush(self):
        self.text = ''
        line = self.getline()
        while line != 'What do you do?':
            line = self.getline()

    def check_events(self):
        for event in open_events:
            try:
                for action in event(self):
                    self.do_action(action)
                    self.read_text()
            except AssertionError:
                pass
        
    def do_action(self, action):
        if DEBUG == 2:
            print('>>', action)
        self.game.stdin.write(action.encode('utf-8') + b'\n')
        self.game.stdin.flush()
        if not self.dry_run:
            self.actions.append(action)

    def play(self):
        if not self.progress:
            self.inventory = set()
        else:
            self.flush()
            for action in self.progress:
                self.do_action(action)
                self.flush()
            go = random.choice(self.saved_exits)
            self.do_action(go)
            self.last_move = go

        try:
            while open_events:
                self.read_text()
                self.check_events()
                go = random.choice(self.exits)
                self.do_action(go)
                self.last_move = go
            print(self.text)
        except EatenByGrue:
            if DEBUG == 1:
                print("You have been eaten by a grue!")
            self.kill()

    def kill(self):
        if self.game:
            self.game.kill()
            self.game = None


import sys
if __name__ == '__main__':
    try:
        p = Player(sys.argv[-1])
        p.play()
        while open_events:
            if DEBUG == 1:
                print('Restarting ...')
            p.reset()
            p.play()
        p.kill()
    except KeyboardInterrupt:
        p.kill()
