import datetime, decimal

try:
    _range = xrange
except NameError:
    unicode = str
    _range = range
    basestring = str
    unichr = chr

def load(f):
    """Returns a dictionary containing the named file parsed as toml."""
    if isinstance(f, basestring):
        with open(f) as ffile:
            return loads(ffile.read())
    elif isinstance(f, list):
        for l in f:
            if not isinstance(l, basestring):
                raise Exception("Load expects a list to contain filenames only")
        d = []
        for l in f:
            d.append(load(l))
        r = {}
        for l in d:
            toml_merge_dict(r, l)
        return r
    elif f.read:
        return loads(f.read())
    else:
        raise Exception("You can only load a file descriptor, filename or list")

def loads(s):
    """Returns a dictionary containing s, a string, parsed as toml."""
    implicitgroups = []
    retval = {}
    currentlevel = retval
    if isinstance(s, basestring):
        try:
            s.decode('utf8')
        except AttributeError:
            pass
        sl = list(s)
        openarr = 0
        openstring = False
        arrayoftables = True
        beginline = True
        keygroup = False
        delnum = 1
        for i in range(len(sl)):
            if sl[i] == '"':
                oddbackslash = False
                try:
                    k = 1
                    j = sl[i-k]
                    oddbackslash = False
                    while j == '\\':
                        oddbackslash = not oddbackslash
                        k += 1
                        j = sl[i-k]
                except IndexError:
                    pass
                if not oddbackslash:
                    openstring = not openstring
            if keygroup and (sl[i] == ' ' or sl[i] == '\t'):
                keygroup = False
            if arrayoftables and (sl[i] == ' ' or sl[i] == '\t'):
                arrayoftables = False
            if sl[i] == '#' and not openstring and not keygroup and not arrayoftables:
                j = i
                while sl[j] != '\n':
                    sl.insert(j, ' ')
                    sl.pop(j+1)
                    j += 1
            if sl[i] == '[' and not openstring and not keygroup and not arrayoftables:
                if beginline:
                    if sl[i+1] == '[':
                        arrayoftables = True
                    else:
                        keygroup = True
                else:
                    openarr += 1
            if sl[i] == ']' and not openstring and not keygroup and not arrayoftables:
                if keygroup:
                    keygroup = False
                elif arrayoftables:
                    if sl[i-1] == ']':
                        arrayoftables = False
                else:
                    openarr -= 1
            if sl[i] == '\n':
                if openstring:
                    raise Exception("Unbalanced quotes")
                if openarr:
                    sl.insert(i, ' ')
                    sl.pop(i+1)
                else:
                    beginline = True
            elif beginline and sl[i] != ' ' and sl[i] != '\t':
                beginline = False
                keygroup = True
        s = ''.join(sl)
        s = s.split('\n')
    else:
        raise Exception("What exactly are you trying to pull?")
    for line in s:
        line = line.strip()
        if line == "":
            continue
        if line[0] == '[':
            arrayoftables = False
            if line[1] == '[':
                arrayoftables = True
                line = line[2:].split(']]', 1)
            else:
                line = line[1:].split(']', 1)
            if line[1].strip() != "":
                raise Exception("Key group not on a line by itself.")
            line = line[0]
            if '[' in line:
                raise Exception("Key group name cannot contain '['")
            if ']' in line:
                raise Exception("Key group name cannot contain']'")
            groups = line.split('.')
            currentlevel = retval
            for i in range(len(groups)):
                group = groups[i]
                if group == "":
                    raise Exception("Can't have a keygroup with an empty name")
                try:
                    currentlevel[group]
                    if i == len(groups) - 1:
                        if group in implicitgroups:
                            implicitgroups.remove(group)
                            if arrayoftables:
                                raise Exception("An implicitly defined table can't be an array")
                        elif arrayoftables:
                            currentlevel[group].append({})
                        else:
                            raise Exception("What? "+group+" already exists?"+str(currentlevel))
                except TypeError:
                    if i != len(groups) - 1:
                        implicitgroups.append(group)
                    currentlevel = currentlevel[0]
                    if arrayoftables:
                        currentlevel[group] = [{}]
                    else:
                        currentlevel[group] = {}
                except KeyError:
                    if i != len(groups) - 1:
                        implicitgroups.append(group)
                    currentlevel[group] = {}
                    if i == len(groups) - 1 and arrayoftables:
                        currentlevel[group] = [{}]
                currentlevel = currentlevel[group]
                if arrayoftables:
                    try:
                        currentlevel = currentlevel[-1]
                    except KeyError:
                        pass
        elif "=" in line:
            i = 1
            pair = line.split('=', i)
            l = len(line)
            while pair[-1][0] != ' ' and pair[-1][0] != '\t' and pair[-1][0] != '"' and pair[-1][0] != '[' and pair[-1] != 'true' and pair[-1] != 'false':
                try:
                    float(pair[-1])
                    break
                except ValueError:
                    try:
                        datetime.datetime.strptime(pair[-1], "%Y-%m-%dT%H:%M:%SZ")
                        break
                    except ValueError:
                        i += 1
                        pair = line.split('=', i)
            newpair = []
            newpair.append('='.join(pair[:-1]))
            newpair.append(pair[-1])
            pair = newpair
            pair[0] = pair[0].strip()
            pair[1] = pair[1].strip()
            value, vtype = load_value(pair[1])
            try:
                currentlevel[pair[0]]
                raise Exception("Duplicate keys!")
            except KeyError:
                currentlevel[pair[0]] = value
    return retval

def load_value(v):
    if v == 'true':
        return (True, "bool")
    elif v == 'false':
        return (False, "bool")
    elif v[0] == '"':
        testv = v[1:].split('"')
        closed = False
        for tv in testv:
            if tv == '':
                closed = True
            else:
                oddbackslash = False
                try:
                    i = -1
                    j = tv[i]
                    while j == '\\':
                        oddbackslash = not oddbackslash
                        i -= 1
                        j = tv[i]
                except IndexError:
                    pass
                if not oddbackslash:
                    if closed:
                        raise Exception("Stuff after closed string. WTF?")
                    else:
                        closed = True
        escapes = ['0', 'b', 'f', '/', 'n', 'r', 't', '"', '\\']
        escapedchars = ['\0', '\b', '\f', '/', '\n', '\r', '\t', '\"', '\\']
        escapeseqs = v.split('\\')[1:]
        backslash = False
        for i in escapeseqs:
            if i == '':
                backslash = not backslash
            else:
                if i[0] not in escapes and i[0] != 'u' and not backslash:
                    raise Exception("Reserved escape sequence used")
                if backslash:
                    backslash = False
        if "\\u" in v:
            hexchars = ['0', '1', '2', '3', '4', '5', '6', '7',
                        '8', '9', 'a', 'b', 'c', 'd', 'e', 'f']
            hexbytes = v.split('\\u')
            newv = hexbytes[0]
            hexbytes = hexbytes[1:]
            for hx in hexbytes:
                hxb = ""
                try:
                    if hx[0].lower() in hexchars:
                        hxb += hx[0].lower()
                        if hx[1].lower() in hexchars:
                            hxb += hx[1].lower()
                        if hx[2].lower() in hexchars:
                            hxb += hx[2].lower()
                            if hx[3].lower() in hexchars:
                                hxb += hx[3].lower()
                except IndexError:
                    if len(hxb) != 2:
                        raise Exception("Invalid escape sequence")
                if len(hxb) != 4 and len(hxb) != 2:
                    raise Exception("Invalid escape sequence")
                newv += unichr(int(hxb, 16))
                newv += unicode(hx[len(hxb):])
            v = newv
        for i in range(len(escapes)):
            v = v.replace("\\"+escapes[i], escapedchars[i])
            # (where (n) signifies a member of escapes:
            # undo (\\)(\\)(n) -> (\\)(\n)
            v = v.replace("\\"+escapedchars[i], "\\\\"+escapes[i])
        return (v[1:-1], "str")
    elif v[0] == '[':
        return (load_array(v), "array")
    elif len(v) == 20 and v[-1] == 'Z':
        if v[10] == 'T':
            return (datetime.datetime.strptime(v, "%Y-%m-%dT%H:%M:%SZ"), "date")
        else:
            raise Exception("Wait, what?")
    else:
        itype = "int"
        digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
        neg = False
        if v[0] == '-':
            neg = True
            v = v[1:]
        if '.' in v:
            if v.split('.', 1)[1] == '':
                raise Exception("This float is missing digits after the point")
            if v[0] not in digits:
                raise Exception("This float doesn't have a leading digit")
            v = float(v)
            itype = "float"
        else:
            v = int(v)
        if neg:
            return (0 - v, itype)
        return (v, itype)


def load_array(a):
    atype = None
    retval = []
    a = a.strip()
    if '[' not in a[1:-1]:
        strarray = False
        tmpa = a[1:-1].strip()
        if tmpa != '' and tmpa[0] == '"':
            strarray = True
        a = a[1:-1].split(',')
        b = 0
        if strarray:
            while b < len(a) - 1:
                while a[b].strip()[-1] != '"' and a[b+1].strip()[0] != '"':
                    a[b] = a[b] + ',' + a[b+1]
                    if b < len(a) - 2:
                        a = a[:b+1] + a[b+2:]
                    else:
                        a = a[:b+1]
                b += 1
    else:
        al = list(a[1:-1])
        a = []
        openarr = 0
        j = 0
        for i in range(len(al)):
            if al[i] == '[':
                openarr += 1
            elif al[i] == ']':
                openarr -= 1
            elif al[i] == ',' and not openarr:
                a.append(''.join(al[j:i]))
                j = i+1
        a.append(''.join(al[j:]))
    for i in range(len(a)):
        a[i] = a[i].strip()
        if a[i] != '':
            nval, ntype = load_value(a[i])
            if atype:
                if ntype != atype:
                    raise Exception("Not a homogeneous array")
            else:
                atype = ntype
            retval.append(nval)
    return retval

def dump(o, f):
    """Writes out to f the toml corresponding to o. Returns said toml."""
    if f.write:
        d = dumps(o)
        f.write(d)
        return d
    else:
        raise Exception("You can only dump an object to a file descriptor")

def dumps(o):
    """Returns a string containing the toml corresponding to o, a dictionary"""
    retval = ""
    addtoretval, sections = dump_sections(o, "")
    retval += addtoretval
    while sections != {}:
        newsections = {}
        for section in sections:
            addtoretval, addtosections = dump_sections(sections[section], section)
            if addtoretval:
                retval += "["+section+"]\n"
                retval += addtoretval
            for s in addtosections:
                newsections[section+"."+s] = addtosections[s]
        sections = newsections
    return retval

def dump_sections(o, sup):
    retstr = ""
    if sup != "" and sup[-1] != ".":
        sup += '.'
    retdict = {}
    arraystr = ""
    for section in o:
        if not isinstance(o[section], dict):
            arrayoftables = False
            if isinstance(o[section], list):
                for a in o[section]:
                    if isinstance(a, dict):
                        arrayoftables = True
            if arrayoftables:
                for a in o[section]:
                    arraytabstr = ""
                    arraystr += "[["+sup+section+"]]\n"
                    s, d = dump_sections(a, sup+section)
                    if s:
                        if s[0] == "[":
                            arraytabstr += s
                        else:
                            arraystr += s
                    while d != {}:
                        newd = {}
                        for dsec in d:
                            s1, d1 = dump_sections(d[dsec], sup+section+dsec)
                            if s1:
                                arraytabstr += "["+sup+section+"."+dsec+"]\n"
                                arraytabstr += s1
                            for s1 in d1:
                                newd[dsec+"."+s1] = d1[s1]
                        d = newd
                    arraystr += arraytabstr
            else:
                retstr += section + " = " + str(dump_value(o[section])) + '\n'
        else:
            retdict[section] = o[section]
    retstr += arraystr
    return (retstr, retdict)

def dump_value(v):
    if isinstance(v, list):
        t = []
        retval = "["
        for u in v:
            t.append(dump_value(u))
        while t != []:
            s = []
            for u in t:
                if isinstance(u, list):
                    for r in u:
                        s.append(r)
                else:
                    retval += " " + str(u) + ","
            t = s
        retval += "]"
        return retval
    if isinstance(v, (str, unicode)):
        escapes = ['\\', '0', 'b', 'f', '/', 'n', 'r', 't', '"']
        escapedchars = ['\\', '\0', '\b', '\f', '/', '\n', '\r', '\t', '\"']
        for i in range(len(escapes)):
            v = v.replace(escapedchars[i], "\\"+escapes[i])
        return str('"'+v+'"')
    if isinstance(v, bool):
        return str(v).lower()
    if isinstance(v, datetime.datetime):
        return v.isoformat()[:19]+'Z'
    if isinstance(v, float):
        return '{0:f}'.format(decimal.Decimal(str(v)))
    return v

def toml_merge_dict(a, b):
    for k in a:
        if isinstance(a[k], dict):
            try:
                b[k]
            except KeyError:
                continue
            if isinstance(b[k], dict):
                b[k] = toml_merge_dict(a[k], b[k])
            else:
                raise Exception("Can't merge dict and nondict in toml object")
    a.update(b)
    return a
