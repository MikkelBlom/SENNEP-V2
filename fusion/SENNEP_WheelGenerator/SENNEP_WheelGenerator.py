# SENNEP V2 - mecanum wheel generator for Autodesk Fusion.
#
# Everything is 3D printed, no metal parts:
#   - rollers (PETG core + TPU sleeve) run on printed pin stubs that stick out of both rim halves
#   - Rim A (inner, motor side) and Rim B (outer) twist together around the rollers
#   - the hex axle goes through both rims, which locks the twist (no extra lock part)
#   - the axle has a head on the outside, a round section for the 17 mm pillow block
#     bearing and a coupling bore for the 8 mm motor shaft (M3 set screw + heat insert)
#
# HOW TO USE (in Fusion): edit the "Wheel_..." / "Axle_..." numbers in
# Modify > Change Parameters, then run this script (Utilities > Scripts and Add-Ins).
# The wheel component is rebuilt in place, so every copy of it updates.
import adsk.core, adsk.fusion, math, traceback

TARGET = '11 Wheel Lab'          # component that is (re)built
PLACE = (450.0, 0.0)             # where the component is created the first time (x, y in mm)
HAND = 1                         # +1 / -1 = the two mecanum hands

# name: (default, unit, comment). Read from the design's user parameters, created if missing.
PARAMS = {
    'Wheel_OD':        (106.2, 'mm', 'Ydre diameter over sleeves'),
    'Wheel_Rollers':   (9, '', 'Antal ruller'),
    'Wheel_Angle':     (45, 'deg', 'Rullevinkel til hjulaksen'),
    'Wheel_RollerDia': (21.5, 'mm', 'Rulle-diameter paa midten (inkl. evt. sleeve)'),
    'Wheel_Sleeve':    (1.2, 'mm', 'TPU-sleeve tykkelse (0 = ingen sleeve; kernen alene skal stadig stikke ud over faelgen)'),
    'Wheel_Overlap':   (4, '', 'Ekstra rulledaekning i procent'),
    'Wheel_Clear':     (1.5, 'mm', 'Frigang rulle-faelg'),
    'Wheel_PinDia':    (6.5, 'mm', 'Printet pindestump (i faelgen)'),
    'Wheel_PinClear':  (0.3, 'mm', 'Radial frigang pind-rulle'),
    'Wheel_PinRoot':   (1.0, 'mm', 'Afrunding/flare ved pindens rod (styrke)'),
    'Wheel_StubA':     (8.0, 'mm', 'Pindens indgreb i rullen fra Rim A'),
    'Wheel_StubB':     (6.0, 'mm', 'Pindens indgreb i rullen fra Rim B (kort pga. skruebevaegelsen)'),
    'Wheel_EndGap':    (0.4, 'mm', 'Aksial frigang rulleende-trykring (pr. ende)'),
    'Wheel_ThrustRing': (2.5, 'mm', 'Trykringens bredde ud over pinden (rullen loeber mod den, ikke mod pinderoden)'),
    'Wheel_BoreChamfer': (0.6, 'mm', 'Affasning i rullens boring (lille = stor flad ende til print)'),
    'Wheel_StubBClear': (0.3, 'mm', 'Rim B-pindenes tilspidsning (radialt mindre ved spidsen, fuld ved roden)'),
    'Wheel_Boss':      (5.0, 'mm', 'Laengde af pindeholder i faelgen'),
    'Wheel_RimMargin': (3.5, 'mm', 'Faelgens radius under ydre diameter (= hvor meget rullerne stikker ud)'),
    'Wheel_Windows':   (6, '', 'Letvaegtsvinduer i navet (0 = ingen)'),
    'Wheel_SidePlate': (3.0, 'mm', 'Tykkelse af faelgens sideskive'),
    'Wheel_FaceDepth': (5.0, 'mm', 'Ekstra dybde paa sidefladen (udad)'),
    'Wheel_FaceChamfer': (5.0, 'mm', 'Skraa kant fra faelgkant ind mod sidefladen'),
    'Wheel_FaceBlend': (8.0, 'mm', 'Afrunding mellem skraa kant og sideflade'),
    'Wheel_WindowWeb': (9.0, 'mm', 'Materiale mellem navvinduerne'),
    'Wheel_WindowOuter': (35.0, 'mm', 'Navvinduernes ydre radius'),
    'Wheel_WindowInnerFillet': (3.0, 'mm', 'Afrunding hvor vinduerne munder ud inde i hjulet'),
    'Wheel_Bolts':     (0, '', 'Ekstra M3-skruer indefra (0 = kun gevindmoetrik)'),
    'Wheel_HubTube':   (30.0, 'mm', 'Rundt navroer paa Rim A (gaar gennem Rim B)'),
    'Wheel_Keys':      (3, '', 'Antal skraa riller (foelger pindenes 45-graders skruelinje)'),
    'Wheel_KeyWidth':  (5.0, 'mm', 'Rillebredde'),
    'Wheel_KeyHeight': (2.5, 'mm', 'Rillehoejde'),
    'Wheel_KeyClear':  (0.1, 'mm', 'Frigang i sporet (pr. side) - lille = stramt, ingen slingren'),
    'Wheel_TubeClear': (0.15, 'mm', 'Radial frigang navroer i Rim B'),
    'Wheel_BoltLen':   (20.0, 'mm', 'Skruelaengde'),
    'Wheel_BoltClear': (3.4, 'mm', 'Frigangshul til skruen'),
    'Wheel_BoltHead':  (6.2, 'mm', 'Forsænkning til skruehoved'),
    'Wheel_InsertDia': (4.0, 'mm', 'Hul til gevindindsats (i Rim B)'),
    'Wheel_InsertDepth': (6.0, 'mm', 'Dybde til gevindindsats'),
    'Wheel_InnerFillet': (2.0, 'mm', 'Afrunding inde i hjulet (pindeholdere/skive)'),
    'Wheel_Fillet':    (1.5, 'mm', 'Afrunding af ydre kanter'),
    'Wheel_RollerEndFillet': (0.4, 'mm', 'Lille afrunding paa rulleenderne (flad ende til print)'),
    'Axle_HexAF':      (18.0, 'mm', 'Sekskant (noeglevidde) gennem hjulet'),
    'Axle_HexClear':   (0.2, 'mm', 'Frigang sekskant-hul (total over noeglevidden) - lille = stramt'),
    'Axle_HeadDia':    (42.0, 'mm', 'Akselhoved paa hjulets yderside (skal daekke hullet i Rim B)'),
    'Axle_HeadThk':    (4.0, 'mm', 'Akselhoved tykkelse'),
    'Axle_ToBearing':  (12.7, 'mm', 'Hjulets inderside til lejet (= laengde af afstandsroeret)'),
    'Axle_SpacerDia':  (26.0, 'mm', 'Afstandsrør mod lejets inderring'),
    'Spacer_Lock':     (1, '', 'Bajonetlaas mellem afstandsroer og Rim A (0 = ingen)'),
    'Spacer_LockDepth': (7.0, 'mm', 'Hvor dybt afstandsroeret gaar ind i Rim A'),
    'Spacer_Clear':    (0.35, 'mm', 'Radial frigang afstandsroer/tapper i Rim A (skal kunne drejes)'),
    'Spacer_LockClear': (0.15, 'mm', 'Aksial frigang tap-kant (lille = laaser stramt)'),
    'Spacer_LugH':     (2.5, 'mm', 'Tappens hoejde (radialt = aksialt, trekantet 45 grader)'),
    'Spacer_LugW':     (6.0, 'mm', 'Tappens bredde'),
    'Axle_BearingDia': (16.9, 'mm', 'Akslens diameter i lejet (lejeboring 17 minus FDM-frigang)'),
    'Axle_BearingW':   (19.5, 'mm', 'Lejets bredde'),
    'Axle_CouplingL':  (19.0, 'mm', 'Kobling efter lejet (mod motoren)'),
    'Motor_ShaftDia':  (8.0, 'mm', 'Motoraksel'),
    'Motor_ShaftFlat': (0.5, 'mm', 'D-flade dybde (0 = rund)'),
    'Motor_ShaftDepth': (18.0, 'mm', 'Hvor langt motorakslen gaar ind'),
    'Motor_ShaftClear': (0.2, 'mm', 'Radial frigang om motorakslen (printede huller bliver mindre)'),
    'Axle_SetScrewHole': (3.4, 'mm', 'Tvaerhul til pinolskruen (frigang M3)'),
    'Axle_NutAF':      (5.5, 'mm', 'Noeglevidde paa den indfangede moetrik (M3 = 5.5)'),
    'Axle_NutThk':     (2.4, 'mm', 'Moetrikkens tykkelse (M3 = 2.4)'),
    'Axle_NutClear':   (0.2, 'mm', 'Frigang i moetriklommen (pr. maal)'),
}

COLORS = {  # appearance name: RGB
    'SENNEP Rim': (225, 222, 212),
    'SENNEP Roller Core': (214, 150, 40),
    'SENNEP Sleeve': (35, 35, 35),
    'SENNEP Axle': (120, 124, 130),
}

MM = 0.1


def V(x, y, z):
    return adsk.core.Vector3D.create(x, y, z)


def Pt(x, y, z):
    return adsk.core.Point3D.create(x * MM, y * MM, z * MM)


def read_params(des):
    up = des.userParameters
    out = {}
    for name, (val, unit, cmt) in PARAMS.items():
        prm = up.itemByName(name)
        if prm is None:
            expr = ('%g %s' % (val, unit)).strip()
            prm = up.add(name, adsk.core.ValueInput.createByString(expr), unit, cmt)
        if unit == 'mm':
            out[name] = prm.value / MM
        elif unit == 'deg':
            out[name] = math.degrees(prm.value)
        else:
            out[name] = prm.value
    out['Wheel_Rollers'] = int(round(out['Wheel_Rollers']))
    out['Wheel_Windows'] = int(round(out['Wheel_Windows']))
    out['Wheel_Bolts'] = int(round(out['Wheel_Bolts']))
    out['Wheel_Keys'] = int(round(out['Wheel_Keys']))
    out['Spacer_Lock'] = int(round(out['Spacer_Lock']))
    return out


def appearance(des, name):
    a = des.appearances.itemByName(name)
    if a:
        return a
    app = adsk.core.Application.get()
    base = None
    for libname in ('Fusion Appearance Library', 'Fusion 360 Appearance Library'):
        lib = app.materialLibraries.itemByName(libname)
        if lib:
            for i in range(lib.appearances.count):
                if 'Plastic' in lib.appearances.item(i).name and 'Matte' in lib.appearances.item(i).name:
                    base = lib.appearances.item(i)
                    break
        if base:
            break
    if base is None:
        return None
    a = des.appearances.addByCopy(base, name)
    rgb = COLORS[name]
    for i in range(a.appearanceProperties.count):
        prop = a.appearanceProperties.item(i)
        if prop.objectType == adsk.core.ColorProperty.classType() and prop.name in ('Color', 'Base Color', 'Diffuse'):
            prop.value = adsk.core.Color.create(rgb[0], rgb[1], rgb[2], 255)
            break
    return a


def try_fillet(comp, body, radius_mm, pick):
    """Fillet the edges chosen by pick(edge); silently skip if Fusion can't do it."""
    if radius_mm <= 0:
        return 0
    edges = adsk.core.ObjectCollection.create()
    for e in body.edges:
        if pick(e):
            edges.add(e)
    if edges.count == 0:
        return 0
    try:
        fi = comp.features.filletFeatures.createInput()
        fi.addConstantRadiusEdgeSet(edges, adsk.core.ValueInput.createByReal(radius_mm * MM), True)
        comp.features.filletFeatures.add(fi)
        return edges.count
    except Exception:
        return -edges.count


def is_circle(e):
    return e.geometry.objectType == adsk.core.Circle3D.classType()


def build():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    root = des.rootComponent
    tbm = adsk.fusion.TemporaryBRepManager.get()
    Q = read_params(des)
    log = []

    N = Q['Wheel_Rollers']
    R = Q['Wheel_OD'] / 2
    rmax = Q['Wheel_RollerDia'] / 2
    Ra = R - rmax
    a = math.radians(Q['Wheel_Angle'])
    sa, ca = math.sin(a), math.cos(a)
    SLV, CLR = Q['Wheel_Sleeve'], Q['Wheel_Clear']
    pin_r = Q['Wheel_PinDia'] / 2
    bore_r = pin_r + Q['Wheel_PinClear']

    def r_out(s):
        rho = math.sqrt(Ra ** 2 + (s * sa) ** 2)
        wd = s * sa * sa / rho
        return (R - rho) / math.sqrt(1 - wd * wd)

    phi = math.pi / N * (1 + Q['Wheel_Overlap'] / 100.0)
    s_end = Ra * math.tan(phi) / sa
    r_end = r_out(s_end)
    wall_end = r_end - SLV - bore_r
    log.append('Ruller: %d stk, laengde %.1f mm, diameter %.1f (midt) / %.1f (ende)' % (N, 2 * s_end, 2 * rmax, 2 * r_end))
    log.append('Rullevaeg ved enden: %.2f mm%s' % (wall_end, '  << FOR TYND' if wall_end < 1.8 else ''))

    # ---------- target component, rebuilt in place ----------
    tgt = des.allComponents.itemByName(TARGET)
    if tgt is None:
        m = adsk.core.Matrix3D.create()
        m.translation = V(PLACE[0] * MM, PLACE[1] * MM, R * MM)
        tgt = root.occurrences.addNewComponent(m).component
        tgt.name = TARGET
    for coll in (tgt.motionLinks, tgt.joints):  # joints from the previous build first
        for i in range(coll.count - 1, -1, -1):
            coll.item(i).deleteMe()
    for o in list(tgt.occurrences):
        o.deleteMe()
    for b in list(tgt.bRepBodies):
        b.deleteMe()
    tocc = [o for o in root.allOccurrences if o.component == tgt][0]

    # ---------- roller frames ----------
    mats = []
    for i in range(N):
        th = 2 * math.pi * i / N
        er = V(0, math.cos(th), math.sin(th))
        et = V(0, -math.sin(th), math.cos(th))
        d = V(ca, HAND * sa * et.y, HAND * sa * et.z)
        d.normalize()
        m = adsk.core.Matrix3D.create()
        m.setWithCoordinateSystem(Pt(0, Ra * er.y, Ra * er.z), d, er, d.crossProduct(er))
        mats.append((m, d, er))

    def along(c0, d, s):
        return adsk.core.Point3D.create(c0.x + d.x * s * MM, c0.y + d.y * s * MM, c0.z + d.z * s * MM)

    # ---------- master roller (parametric revolve) ----------
    rcOcc = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create())
    rc = rcOcc.component
    rc.name = TARGET + ' Roller (print 1 per roller)'
    n = 33
    ss = [-s_end + 2 * s_end * i / (n - 1) for i in range(n)]

    def spline(sk, pts):
        oc = adsk.core.ObjectCollection.create()
        for (x, y) in pts:
            oc.add(Pt(x, y, 0))
        return sk.sketchCurves.sketchFittedSplines.add(oc)

    def profile(top_pts, bottom_y=None, bottom_pts=None):
        sk = rc.sketches.add(rc.xYConstructionPlane)
        top = spline(sk, top_pts)
        L = sk.sketchCurves.sketchLines
        if bottom_pts is not None:
            bot = spline(sk, bottom_pts)
            L.addByTwoPoints(top.startSketchPoint, bot.startSketchPoint)
            L.addByTwoPoints(top.endSketchPoint, bot.endSketchPoint)
        else:
            a0 = L.addByTwoPoints(top.startSketchPoint, Pt(top_pts[0][0], bottom_y, 0))
            a1 = L.addByTwoPoints(top.endSketchPoint, Pt(top_pts[-1][0], bottom_y, 0))
            L.addByTwoPoints(a0.endSketchPoint, a1.endSketchPoint)
        return sk.profiles.item(0)

    def revolve(prof):
        ri = rc.features.revolveFeatures.createInput(prof, rc.xConstructionAxis,
                                                    adsk.fusion.FeatureOperations.NewBodyFeatureOperation)
        ri.setAngleExtent(False, adsk.core.ValueInput.createByString('360 deg'))
        return rc.features.revolveFeatures.add(ri).bodies.item(0)

    # core: bore with a chamfered entry at both ends, mating with the flared pin root on the rim
    CH = Q['Wheel_BoreChamfer']  # small lead-in; the pin's root flare ends before the roller, so no deep chamfer needed
    sk = rc.sketches.add(rc.xYConstructionPlane)
    top = spline(sk, [(s, r_out(s) - SLV) for s in ss])
    L = sk.sketchCurves.sketchLines
    pts = [(s_end, bore_r + CH), (s_end - CH, bore_r), (-s_end + CH, bore_r), (-s_end, bore_r + CH)]
    prev = top.endSketchPoint
    for (x, y) in pts:
        ln = L.addByTwoPoints(prev, Pt(x, y, 0))
        prev = ln.endSketchPoint
    L.addByTwoPoints(prev, top.startSketchPoint)
    core = revolve(sk.profiles.item(0))
    sleeve = revolve(profile([(s, r_out(s)) for s in ss], bottom_pts=[(s, r_out(s) - SLV) for s in ss])) if SLV > 0 else None
    ce = s_end + CLR
    cs = [-ce + 2 * ce * i / (n - 1) for i in range(n)]
    clear = revolve(profile([(s, r_out(s) + CLR) for s in cs], bottom_y=0.0))
    clear2 = revolve(profile([(s, r_out(s) + CLR + 0.25) for s in cs], bottom_y=0.0))
    core.name, clear.name = 'Roller Core (PETG)', 'Clearance (tool)'
    if sleeve:
        sleeve.name = 'Roller Sleeve (TPU)'
    clear2.name = 'Clearance+ (tool)'
    f = Q['Wheel_Fillet']
    ref = Q['Wheel_RollerEndFillet']
    try_fillet(rc, core, min(ref, wall_end * 0.4), lambda e: is_circle(e) and e.geometry.radius > (bore_r + CH) * MM * 1.01)
    if sleeve:
        try_fillet(rc, sleeve, min(ref, SLV * 0.45), lambda e: is_circle(e) and e.geometry.radius > (r_end - SLV) * MM * 1.01)
    core = rc.bRepBodies.itemByName('Roller Core (PETG)')
    sleeve = rc.bRepBodies.itemByName('Roller Sleeve (TPU)') if SLV > 0 else None
    clear = rc.bRepBodies.itemByName('Clearance (tool)')
    clear2 = rc.bRepBodies.itemByName('Clearance+ (tool)')
    clear.isLightBulbOn = False
    clear2.isLightBulbOn = False

    # ---------- assembled rollers (copies as bodies) ----------
    rcOcc.createForAssemblyContext(tocc).isLightBulbOn = False
    rset = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create()).component
    rset.name = TARGET + ' Rollers (assembled)'
    bf = rset.features.baseFeatures.add()
    bf.startEdit()
    parts = [core, sleeve] if sleeve else [core]
    for (m, d, er) in mats:
        for src in parts:
            c = tbm.copy(src)
            tbm.transform(c, m)
            rset.bRepBodies.add(c, bf)
    bf.finishEdit()
    for j, b in enumerate(rset.bRepBodies):
        b.name = 'Roller %d %s' % (j // len(parts) + 1, 'Core' if j % len(parts) == 0 else 'Sleeve')

    # ---------- rims ----------
    # Side plate sits just outside the rollers' swept clearance, so it is never notched;
    # the pin bosses run from the roller end straight into the plate (no separate spokes).
    sp = Q['Wheel_SidePlate']
    r_rim = R - Q['Wheel_RimMargin']
    hub_r = Ra - rmax - CLR
    boss_r = pin_r + 2.5
    ax_max = max(s * ca + (r_out(s) + CLR) * sa for s in [ce * i / 40 for i in range(41)])
    plate_in = ax_max + 0.4
    x_out = plate_in + sp
    s0 = s_end + CLR
    s1 = max(s0 + Q['Wheel_Boss'], (plate_in + 0.6 + boss_r * sa) / ca)
    BOSS = s1 - s0
    PR = Q['Wheel_PinRoot']
    hexR = (Q['Axle_HexAF'] + Q['Axle_HexClear']) / 2
    pin_top = math.sqrt(Ra ** 2 + ((s_end + Q['Wheel_EndGap']) * sa) ** 2) + pin_r  # pin root = thrust ring face
    bare = R - SLV - r_rim - Q['Wheel_PinClear']
    log.append('Rullerne stikker %.1f mm ud over faelgen uden sleeve, %.1f mm med (efter pindefrigang)%s' %
               (bare, bare + SLV, '' if bare > 1.2 else '  << FOR LIDT - faelgen roerer gulvet uden sleeves'))
    if pin_top + 1.0 > r_rim:
        log.append('ADVARSEL: pindens rod (r=%.1f) er under 1 mm fra faelgkanten (r=%.1f)' % (pin_top, r_rim))

    def hex_prism(af, x0, x1):
        body = None
        L = abs(x1 - x0)
        for k in range(3):
            ang = math.radians(60 * k)
            obb = adsk.core.OrientedBoundingBox3D.create(Pt((x0 + x1) / 2, 0, 0), V(1, 0, 0),
                                                         V(0, math.cos(ang), math.sin(ang)),
                                                         L * MM, af * MM, 300 * MM)
            slab = tbm.createBox(obb)
            if body is None:
                body = slab
            else:
                tbm.booleanOperation(body, slab, adsk.fusion.BooleanTypes.IntersectionBooleanType)
        return body

    def cyl(x0, x1, r, y=0.0, z=0.0):
        return tbm.createCylinderOrCone(Pt(x0, y, z), r * MM, Pt(x1, y, z), r * MM)

    U, D, I = (adsk.fusion.BooleanTypes.UnionBooleanType, adsk.fusion.BooleanTypes.DifferenceBooleanType,
               adsk.fusion.BooleanTypes.IntersectionBooleanType)

    # Outer face: extra depth with a 45-degree bevel to the rim edge (Mikkel's Extrude + Chamfer edit)
    FD, FCH = Q['Wheel_FaceDepth'], Q['Wheel_FaceChamfer']
    x_face = x_out + FD
    clears, clears2 = [], []
    for (m, d, er) in mats:
        for src, lst in ((clear, clears), (clear2, clears2)):
            c = tbm.copy(src)
            tbm.transform(c, m)
            lst.append(c)

    def half_space(ang, keep_positive, x0, x1):
        n = V(0, -math.sin(ang), math.cos(ang))
        big = 400.0
        sgn = 1 if keep_positive else -1
        ctr = Pt((x0 + x1) / 2, sgn * n.y * big / 2, sgn * n.z * big / 2)
        return tbm.createBox(adsk.core.OrientedBoundingBox3D.create(ctr, V(1, 0, 0), n, abs(x1 - x0) * MM, big * MM, big * MM))

    def kidney(r0, r1, a0, a1, x0, x1):
        """Annular sector between r0..r1 and angles a0..a1, with round ends."""
        w = (r1 - r0) / 2
        rm = (r0 + r1) / 2
        k = cyl(x0, x1, r1)
        tbm.booleanOperation(k, cyl(x0 - 1, x1 + 1, r0), D)
        tbm.booleanOperation(k, half_space(a0, True, x0 - 1, x1 + 1), I)
        tbm.booleanOperation(k, half_space(a1, False, x0 - 1, x1 + 1), I)
        for aa in (a0, a1):
            tbm.booleanOperation(k, cyl(x0, x1, w, rm * math.cos(aa), rm * math.sin(aa)), U)
        return k

    nw_, nb_ = Q['Wheel_Windows'], Q['Wheel_Bolts']
    tube_r = Q['Wheel_HubTube'] / 2
    groove_r = tube_r + Q['Wheel_KeyHeight'] + Q['Wheel_KeyClear']
    rw0_ = max(Q['Axle_HeadDia'] / 2, Q['Axle_SpacerDia'] / 2, groove_r) + 2.5
    rw1_ = Q['Wheel_WindowOuter']
    if groove_r + 3.0 > Q['Axle_HeadDia'] / 2:
        log.append('ADVARSEL: akselhovedet (D%.0f) daekker ikke hullet i Rim B (D%.1f)' % (Q['Axle_HeadDia'], 2 * groove_r))
    rb_ = (rw0_ + hub_r - 2.5) / 2
    win_phase = math.pi / nw_ if nw_ else 0.0   # windows sit between the bolts
    bolt_pos = [(rb_ * math.cos(2 * math.pi * k / nb_), rb_ * math.sin(2 * math.pi * k / nb_)) for k in range(nb_)]
    x_cb = -(Q['Wheel_InsertDepth'] - 0.5) + Q['Wheel_BoltLen']  # screw head seat (Rim A), tip ends 0.5 mm short of the insert bottom
    if nb_:
        log.append('Skruer: %d x M3x%.0f indefra (robot-siden), hovedsaede %.1f mm under Rim A-fladen%s' %
                   (nb_, Q['Wheel_BoltLen'], x_face - x_cb, '  << FOR KORT/LANG' if not (0 < x_cb < x_face) else ''))

    # bayonet: 3 lugs on the spacer enter slots in Rim A's outer face, then a 60-degree turn
    # locks them under a ledge (60 degrees keeps the hex bores aligned for the axle)
    Sr = Q['Axle_SpacerDia'] / 2
    Ld, Lh, Lw = Q['Spacer_LockDepth'], Q['Spacer_LugH'], Q['Spacer_LugW']
    lug_ang = [2 * math.pi * k / 3 for k in range(3)]

    def radial_box(ang, r0, r1, width, x0, x1):
        rm = (r0 + r1) / 2
        return tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
            Pt((x0 + x1) / 2, rm * math.cos(ang), rm * math.sin(ang)),
            V(0, math.cos(ang), math.sin(ang)), V(0, -math.sin(ang), math.cos(ang)),
            (r1 - r0) * MM, width * MM, abs(x1 - x0) * MM))

    # Bayonet, one profile: Rim A prints outer face down, so the socket has a single 45-degree
    # cone running from the lug groove all the way down to the hex; the spacer's triangular lugs
    # and its chamfered end lie on the same line, offset by the clearance. No flat roofs anywhere.
    Sc, Lc = Q['Spacer_Clear'], Q['Spacer_LockClear']
    Rp = Sr + Sc                 # pocket radius
    R2 = Sr + Lh + Sc            # groove radius
    ECH = 1.0                    # chamfer on the spacer's end (gives it a flat ring to print on)
    zf = Ld - Lh - ECH           # depth of the lugs' flat locking face below Rim A's outer face
    zg = zf - Lc                 # depth of the ledge the lugs lock behind

    def cone45(x_base, r_base):
        # 45-degree cone with its base at x_base, narrowing towards the inside of the wheel
        return tbm.createCylinderOrCone(Pt(x_base, 0, 0), r_base * MM, Pt(x_base - r_base + 0.2, 0, 0), 0.2 * MM)

    def bayonet_socket():
        pocket = cyl(x_face - zg, x_face + 1, Rp)
        roof = cone45(x_face - zg, R2)
        tbm.booleanOperation(roof, cyl(x_face - zg - 30, x_face - zg + 0.01, Rp), I)
        cuts = [pocket, roof]
        for ph in lug_ang:
            entry = ph - math.pi / 3
            slot = radial_box(entry, 0, R2, Lw + 2 * Sc, x_face - zg, x_face + 1)
            tbm.booleanOperation(slot, cyl(x_face - zg - 1, x_face + 2, R2), I)
            cuts.append(slot)
            groove = cone45(x_face - zg, R2)
            dl = math.asin(min(0.99, (Lw / 2 + Sc) / (Sr - 0.5)))  # clear the lug corner at its inner radius
            tbm.booleanOperation(groove, half_space(entry - dl, True, x_face - Ld - 30, x_face + 1), I)
            tbm.booleanOperation(groove, half_space(ph + dl, False, x_face - Ld - 30, x_face + 1), I)
            cuts.append(groove)
        return cuts

    def rim(side):
        body = cyl(0, side * x_out, hub_r)
        for (m, d, er) in mats:
            c0 = Pt(0, Ra * er.y, Ra * er.z)
            tbm.booleanOperation(body, tbm.createCylinderOrCone(along(c0, d, side * s0), boss_r * MM,
                                                                along(c0, d, side * s1), boss_r * MM), U)
        tbm.booleanOperation(body, cyl(side * (x_out - sp), side * x_out, r_rim), U)
        bevel = tbm.createCylinderOrCone(Pt(side * (x_out - 0.01), 0, 0), r_rim * MM, Pt(side * x_face, 0, 0), (r_rim - FCH) * MM)
        tbm.booleanOperation(body, bevel, U)
        env = cyl(-side * 1, side * x_out, r_rim)  # trims bosses to the plate + bevel envelope
        tbm.booleanOperation(env, tbm.copy(bevel), U)
        tbm.booleanOperation(body, env, I)
        for c in clears:
            tbm.booleanOperation(body, tbm.copy(c), D)
        for i, (m, d, er) in enumerate(mats):  # printed pin stubs, added after the clearance cut
            c0 = Pt(0, Ra * er.y, Ra * er.z)
            tip = 1.0  # chamfered tip for easy assembly
            # Short stubs at both ends (a long cantilever pin snapped off in the v2.7 test print).
            # Rim B's stub tapers towards the tip: Rim B arrives on a screw path, and a straight
            # pin drifts slightly out of line with the bore until it is fully home.
            # The roller's end runs against a flat thrust ring, never against the pin root
            # (a root flare wedged the bore edge and made the rollers stick).
            sr_ = s_end + Q['Wheel_EndGap']                          # thrust ring face
            st_ = s_end - (Q['Wheel_StubA'] if side > 0 else Q['Wheel_StubB'])   # stub tip
            r_tip = pin_r if side > 0 else pin_r - 2 * Q['Wheel_StubBClear']
            fl = min(0.5, CH - 0.1)                                  # small root flare, inside the bore chamfer
            stub = tbm.createCylinderOrCone(along(c0, d, side * st_), (r_tip - 0.8) * MM, along(c0, d, side * (st_ + tip)), r_tip * MM)
            for prt in (tbm.createCylinderOrCone(along(c0, d, side * (st_ + tip)), r_tip * MM, along(c0, d, side * (sr_ - fl)), pin_r * MM),
                        tbm.createCylinderOrCone(along(c0, d, side * (sr_ - fl)), pin_r * MM, along(c0, d, side * sr_), (pin_r + fl) * MM),
                        tbm.createCylinderOrCone(along(c0, d, side * sr_), (pin_r + Q['Wheel_ThrustRing']) * MM,
                                                 along(c0, d, side * (s0 + 0.3)), (pin_r + Q['Wheel_ThrustRing']) * MM)):
                tbm.booleanOperation(stub, prt, U)
            for j, c in enumerate(clears2):  # keep the flare clear of the neighbouring rollers' space
                if j != i:
                    tbm.booleanOperation(stub, tbm.copy(c), D)
            tbm.booleanOperation(stub, cyl(-side * 5, side * x_face, r_rim), I)  # never through the rim edge (same surface as the rim)
            tbm.booleanOperation(body, stub, U)
        if side > 0:   # round hub tube reaching through Rim B (helical keys are swept on afterwards)
            tbm.booleanOperation(body, cyl(0.5, -(x_face - 0.6), tube_r), U)
        else:          # round hole: Rim B turns while it is screwed down along the pins
            tbm.booleanOperation(body, cyl(1, -x_face - 1, tube_r + Q['Wheel_TubeClear']), D)
        tbm.booleanOperation(body, hex_prism(2 * hexR, -x_face - 50, x_face + 50), D)
        if side > 0 and Q['Spacer_Lock']:
            for cut in bayonet_socket():
                tbm.booleanOperation(body, cut, D)
        for (by, bz) in bolt_pos:
            if side > 0:
                tbm.booleanOperation(body, cyl(-1, x_face + 1, Q['Wheel_BoltClear'] / 2, by, bz), D)
                tbm.booleanOperation(body, cyl(x_cb, x_face + 1, Q['Wheel_BoltHead'] / 2, by, bz), D)
            else:
                tbm.booleanOperation(body, cyl(1, -Q['Wheel_InsertDepth'], Q['Wheel_InsertDia'] / 2, by, bz), D)
        nw = Q['Wheel_Windows']
        if nw:
            rw0 = rw0_
            rw1 = rw1_
            rm = (rw0 + rw1) / 2
            span = 2 * math.pi / nw - (Q['Wheel_WindowWeb'] + (rw1 - rw0)) / rm
            for k in range(nw):
                th = 2 * math.pi * k / nw + win_phase
                if span > 0.05:
                    win = kidney(rw0, rw1, th - span / 2, th + span / 2, -side * 2, side * (x_face + 2))
                else:
                    win = cyl(-side * 2, side * (x_face + 2), (rw1 - rw0) / 2, rm * math.cos(th), rm * math.sin(th))
                tbm.booleanOperation(body, win, D)
        return body

    rim_occs = []
    for side, name in ((1, 'Rim A (motor side)'), (-1, 'Rim B (outer)')):
        comp = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create()).component
        comp.name = TARGET + ' ' + name
        bf = comp.features.baseFeatures.add()
        bf.startEdit()
        comp.bRepBodies.add(rim(side), bf)
        bf.finishEdit()
        comp.bRepBodies.item(0).name = name

        def circ(r, x):
            return lambda e: (e.geometry.objectType in (adsk.core.Circle3D.classType(), adsk.core.Arc3D.classType())
                              and abs(e.geometry.radius / MM - r) < 0.05
                              and abs(e.geometry.center.x / MM - x) < 0.05)

        def on_plane_x(x):
            def pick(e):
                if is_circle(e):
                    return False
                fs = [fc for fc in e.faces]
                planes = [fc for fc in fs if fc.geometry.objectType == adsk.core.Plane.classType()
                          and abs(abs(fc.geometry.normal.x) - 1) < 1e-6 and abs(fc.geometry.origin.x / MM - x) < 0.05]
                if len(planes) != 1:
                    return False
                other = [fc for fc in fs if fc != planes[0]][0].geometry
                if other.objectType == adsk.core.Plane.classType():
                    return False  # window side walls
                if other.objectType == adsk.core.Cylinder.classType() and abs(other.axis.x) > 0.99:
                    return False  # window arcs / hub (axis along the wheel axis)
                return True       # pin bosses (axis tilted with the rollers)
            return pick

        xi = side * (x_out - sp)
        steps = [
            ('rim edge', f, circ(r_rim, side * x_out)),
            ('face blend', Q['Wheel_FaceBlend'], circ(r_rim - FCH, side * x_face)),
            ('hub/plate inside', Q['Wheel_InnerFillet'] * 1.5, circ(hub_r, xi)),
            ('bosses/plate inside', Q['Wheel_InnerFillet'], on_plane_x(xi)),
            ('windows', min(f, 1.0), lambda e: e.geometry.objectType in (adsk.core.Arc3D.classType(), adsk.core.Circle3D.classType())
                                     and abs(abs(e.geometry.center.x / MM) - x_face) < 0.05
                                     and ((math.hypot(e.geometry.center.y, e.geometry.center.z) < 0.01 * MM
                                           and rw0_ - 0.5 < e.geometry.radius / MM < rw1_ + 0.5)
                                          or (math.hypot(e.geometry.center.y, e.geometry.center.z) > 1.0 * MM
                                              and e.geometry.radius / MM < (rw1_ - rw0_) / 2 + 0.5))
                                     and abs(e.geometry.radius / MM - Q['Axle_SpacerDia'] / 2) > 0.1),
        ]
        cap_r = (rw1_ - rw0_) / 2

        def window_inside(e):
            # edges of the window walls, except where they meet the outer face or the mating face
            bb = e.boundingBox
            for xp in (side * x_face, 0.0):
                if abs(bb.minPoint.x / MM - xp) < 0.05 and abs(bb.maxPoint.x / MM - xp) < 0.05:
                    return False
            for fc in e.faces:
                g = fc.geometry
                if g.objectType != adsk.core.Cylinder.classType() or abs(g.axis.x) < 0.99:
                    continue
                off = math.hypot(g.origin.y, g.origin.z) / MM
                r = g.radius / MM
                if off > 1.0 and abs(r - cap_r) < 0.05:
                    return True
                if off < 0.01 and (abs(r - rw1_) < 0.05 or abs(r - rw0_) < 0.05):
                    return True
            return False

        steps.append(('windows inside', Q['Wheel_WindowInnerFillet'], window_inside))
        res = []
        for label, rad, pick in steps:
            k = try_fillet(comp, comp.bRepBodies.item(0), rad, pick)
            res.append('%s %s' % (label, 'ok' if k > 0 else ('FEJL' if k < 0 else '-')))
        log.append('%s fillets: %s' % (name, ', '.join(res)))
        rim_occs.append(comp)

    # ---------- axle (hex through wheel, round through bearing, coupling for the motor shaft) ----------
    xh0 = -x_face - Q['Axle_HeadThk']
    xb0 = x_face + Q['Axle_ToBearing']
    xb1 = xb0 + Q['Axle_BearingW']
    xc1 = xb1 + Q['Axle_CouplingL']
    br = Q['Axle_BearingDia'] / 2
    ax = cyl(xh0, -x_face, Q['Axle_HeadDia'] / 2)
    tbm.booleanOperation(ax, hex_prism(Q['Axle_HexAF'], -x_face - 0.01, xb0), U)
    tbm.booleanOperation(ax, cyl(xb0 - 0.01, xc1, br), U)
    sr = Q['Motor_ShaftDia'] / 2 + Q['Motor_ShaftClear']
    shaft = cyl(xc1 - Q['Motor_ShaftDepth'], xc1 + 1, sr)
    if Q['Motor_ShaftFlat'] > 0:
        # box spans z from -sr to sr - flat, so the bore gets a D-shape matching the shaft flat
        keep = tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
            Pt(xc1 - Q['Motor_ShaftDepth'] / 2 + 0.5, 0, -Q['Motor_ShaftFlat'] / 2), V(1, 0, 0), V(0, 1, 0),
            (Q['Motor_ShaftDepth'] + 1) * MM, 2 * sr * MM, (2 * sr - Q['Motor_ShaftFlat']) * MM))
        tbm.booleanOperation(shaft, keep, I)
    tbm.booleanOperation(ax, shaft, D)
    # radial set screw onto the shaft flat, through a captive nut. The nut slides into a pocket
    # that is open at the coupling end (vertical in print, so no pause), flats against the side
    # walls so it can't turn, and is trapped once the screw goes through it. Its inner face is
    # flush with the bore's D-flat (no paper-thin wall; the nut is wider than the flat, so it
    # can't drop into the bore), which leaves the thickest possible outer wall for the load.
    xs = xc1 - Q['Motor_ShaftDepth'] / 2
    tbm.booleanOperation(ax, tbm.createCylinderOrCone(Pt(xs, 0, 0), Q['Axle_SetScrewHole'] / 2 * MM, Pt(xs, 0, br + 1), Q['Axle_SetScrewHole'] / 2 * MM), D)
    naf = Q['Axle_NutAF'] + Q['Axle_NutClear']
    nth = Q['Axle_NutThk'] + Q['Axle_NutClear']
    z0 = sr - Q['Motor_ShaftFlat']                            # flush with the D-flat
    x0 = xs - Q['Axle_NutAF'] / math.sqrt(3) - Q['Axle_NutClear']   # nut corner rests on the pocket floor
    # the pocket walls run straight down until they meet the bore (no knife-edge slivers)
    zl = math.sqrt(max(sr ** 2 - (naf / 2) ** 2, 0.0)) - 0.05
    tbm.booleanOperation(ax, tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
        Pt((x0 + xc1 + 1) / 2, 0, (zl + z0 + nth) / 2), V(1, 0, 0), V(0, 1, 0),
        (xc1 + 1 - x0) * MM, naf * MM, (z0 + nth - zl) * MM)), D)
    skin = br - z0 - nth
    log.append('Moetriklomme M%.0f: %.1f mm vaeg udenfor moetrikken%s' % (Q['Axle_SetScrewHole'] - 0.4, skin,
               '  << FOR TYND' if skin < 1.2 else ''))
    acomp = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create()).component
    acomp.name = TARGET + ' Axle'
    bf = acomp.features.baseFeatures.add()
    bf.startEdit()
    acomp.bRepBodies.add(ax, bf)
    bf.finishEdit()
    acomp.bRepBodies.item(0).name = 'Axle (print standing on the head)'
    try_fillet(acomp, acomp.bRepBodies.item(0), f, lambda e: is_circle(e) and abs(e.geometry.radius / MM - Q['Axle_HeadDia'] / 2) < 0.05)

    # ---------- helical keys: Rim B screws onto Rim A along exactly the pins' 45-degree path ----------
    # Relative motion of Rim B along its pins is a screw about the wheel axis with
    # d(theta)/dx = HAND * tan(angle) / Ra, so keys/grooves following that helix guide the
    # pins straight into the rollers.
    kh = HAND * math.tan(a) / Ra

    def helix(sk, r, th0, x0, x1, n=30):
        oc = adsk.core.ObjectCollection.create()
        for i in range(n + 1):
            x = x0 + (x1 - x0) * i / n
            th = th0 + kh * x
            oc.add(Pt(x, r * math.cos(th), r * math.sin(th)))
        return sk.sketchCurves.sketchFittedSplines.add(oc)

    def sweep_key(comp, r_in, r_out, width, x0, x1, th0, cut):
        body = comp.bRepBodies.item(0)
        sk = comp.sketches.add(comp.xYConstructionPlane)
        path = helix(sk, (r_in + r_out) / 2, th0, x0, x1)
        rail = helix(sk, r_out, th0, x0, x1)
        pin = comp.constructionPlanes.createInput()
        pin.setByOffset(comp.yZConstructionPlane, adsk.core.ValueInput.createByReal(x0 * MM))
        ps = comp.sketches.add(comp.constructionPlanes.add(pin))
        th = th0 + kh * x0
        pts = []
        for (r, sgn) in ((r_in, -1), (r_out, -1), (r_out, 1), (r_in, 1)):
            mp = Pt(x0, r * math.cos(th) - sgn * width / 2 * math.sin(th), r * math.sin(th) + sgn * width / 2 * math.cos(th))
            pts.append(ps.modelToSketchSpace(mp))
        L = ps.sketchCurves.sketchLines
        lines = [L.addByTwoPoints(pts[0], pts[1])]
        lines.append(L.addByTwoPoints(lines[-1].endSketchPoint, pts[2]))
        lines.append(L.addByTwoPoints(lines[-1].endSketchPoint, pts[3]))
        L.addByTwoPoints(lines[-1].endSketchPoint, lines[0].startSketchPoint)
        op = (adsk.fusion.FeatureOperations.CutFeatureOperation if cut
              else adsk.fusion.FeatureOperations.JoinFeatureOperation)
        si = comp.features.sweepFeatures.createInput(ps.profiles.item(0), comp.features.createPath(path, False), op)
        si.guideRail = comp.features.createPath(rail, False)
        si.participantBodies = [body]
        comp.features.sweepFeatures.add(si)

    nk = Q['Wheel_Keys']
    KW, KH, KC = Q['Wheel_KeyWidth'], Q['Wheel_KeyHeight'], Q['Wheel_KeyClear']
    res = []
    for k in range(nk):
        th0 = 2 * math.pi * k / nk + math.pi / max(nk, 1)
        try:
            sweep_key(rim_occs[0], tube_r - 0.5, tube_r + KH, KW, 0.5, -(x_face - 0.6), th0, False)
            sweep_key(rim_occs[1], tube_r - 0.5 - KC, groove_r, KW + 2 * KC, 1.0, -(x_face + 1.0), th0, True)
            res.append('ok')
        except Exception as ex:
            res.append('FEJL (%s)' % str(ex)[:50])
    log.append('Skraa riller (%d stk, %.0f grader drejning ved samling): %s' %
               (nk, math.degrees(abs(kh) * (x_face - 0.6)), ', '.join(res)))

    # ---------- spacer tube (separate part so Rim A prints flat on its outer face) ----------
    sp_t = cyl(x_face, x_face + Q['Axle_ToBearing'], Sr)
    if Q['Spacer_Lock']:
        tbm.booleanOperation(sp_t, cyl(x_face - Ld, x_face + 0.01, Sr), U)
        for ph in lug_ang:
            tbm.booleanOperation(sp_t, radial_box(ph, Sr - 0.5, Sr + Lh, Lw, x_face - Ld, x_face - zf), U)
        # one 45-degree line shapes the lugs (triangles) and the end chamfer
        trim = cone45(x_face - zf, Sr + Lh)
        tbm.booleanOperation(trim, cyl(x_face - zf, x_face + Q['Axle_ToBearing'] + 1, Sr + Lh), U)
        tbm.booleanOperation(sp_t, trim, I)
    tbm.booleanOperation(sp_t, hex_prism(2 * hexR, x_face - Ld - 1, x_face + Q['Axle_ToBearing'] + 1), D)
    scomp = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create()).component
    scomp.name = TARGET + ' Spacer'
    bf = scomp.features.baseFeatures.add()
    bf.startEdit()
    scomp.bRepBodies.add(sp_t, bf)
    bf.finishEdit()
    scomp.bRepBodies.item(0).name = 'Spacer (print standing)'
    try_fillet(scomp, scomp.bRepBodies.item(0), 0.6, lambda e: is_circle(e) and abs(e.geometry.radius / MM - Sr) < 0.05
               and abs(e.geometry.center.x / MM - x_face - Q['Axle_ToBearing']) < 0.05)

    # ---------- colours ----------
    def paint(comp, name):
        ap = appearance(des, name)
        if ap:
            for b in comp.bRepBodies:
                b.appearance = ap
    for comp in rim_occs + [scomp]:
        paint(comp, 'SENNEP Rim')
    paint(acomp, 'SENNEP Axle')
    ap_core, ap_slv = appearance(des, 'SENNEP Roller Core'), appearance(des, 'SENNEP Sleeve')
    for comp in (rset, rc):
        for b in comp.bRepBodies:
            if 'Sleeve' in b.name and ap_slv:
                b.appearance = ap_slv
            elif 'Core' in b.name and ap_core:
                b.appearance = ap_core

    # ---------- quick test prints: the fits, without printing a whole wheel ----------
    tcomp = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create()).component
    tcomp.name = TARGET + ' Testprint (hurtigtest)'
    xt = x_face - 0.6
    pieces = [('Test RimA spacer socket', rim_occs[0], cyl(x_face - Ld - 6, x_face + 1, R2 + 5)),
              ('Test RimA tube end', rim_occs[0], cyl(-xt - 1, -xt + 10, groove_r + 1)),
              ('Test RimB hub', rim_occs[1], cyl(-x_face - 1, -x_face + 10, groove_r + 5)),
              ('Test Axle hex', acomp, cyl(xh0 - 1, -x_face + 12, Q['Axle_HeadDia']))]
    bf = tcomp.features.baseFeatures.add()
    bf.startEdit()
    for label, comp, keep in pieces:
        b = tbm.copy(comp.bRepBodies.item(0))
        tbm.booleanOperation(b, keep, I)
        tcomp.bRepBodies.add(b, bf)
    bf.finishEdit()
    for j, (label, comp, keep) in enumerate(pieces):
        tcomp.bRepBodies.item(j).name = label
    paint(tcomp, 'SENNEP Rim')
    [o for o in tgt.occurrences if o.component == tcomp][0].createForAssemblyContext(tocc).isLightBulbOn = False

    # ---------- checks ----------
    mm_ = app.measureManager
    rsetOcc = [o for o in tgt.occurrences if o.component == rset][0].createForAssemblyContext(tocc)
    sl = [b for b in rsetOcc.bRepBodies if b.name.endswith('Sleeve' if SLV > 0 else 'Core')]
    gap_rr = min(mm_.measureMinimumDistance(sl[i], sl[(i + 1) % len(sl)]).value / MM for i in range(len(sl)))
    rims = [o.createForAssemblyContext(tocc) for o in tgt.occurrences if 'Rim' in o.component.name]
    gap_rim = min(mm_.measureMinimumDistance(s, rb).value / MM for s in sl for ro in rims for rb in ro.bRepBodies)
    log.append('Frigang rulle-rulle %.2f mm, rulle-faelg %.2f mm (maal >= %.1f)' % (gap_rr, gap_rim, CLR))
    log.append('Hjulbredde %.1f mm, faelg-radius %.1f, nav-radius %.1f, pind %.1f mm' % (2 * x_face, r_rim, hub_r, 2 * pin_r))
    log.append('Aksel: hoved %.0f mm, sekskant %.0f mm, lejesektion %.0f mm, total laengde %.1f mm' %
               (Q['Axle_HeadDia'], Q['Axle_HexAF'], Q['Axle_BearingDia'], xc1 - xh0))
    travel = x_face  # how far Rim B is pulled out in the test / joint range
    turn = math.degrees(kh * travel)

    # ---------- assembly test: pull Rim B out along the screw path, look for collisions ----------
    try:
        rimB = [o for o in rims if 'Rim B' in o.component.name][0].bRepBodies.item(0)
        rimA = [o for o in rims if 'Rim A' in o.component.name][0].bRepBodies.item(0)
        obst = [tbm.copy(rimA)] + [tbm.copy(b) for b in rsetOcc.bRepBodies]
        ctr = tocc.transform2.translation
        worst = (0.0, 0.0)
        # fine steps where the short pins enter the rollers, coarse after that
        dxs = [0.25 * k for k in range(1, 33)] + [8.0 + 2.0 * k for k in range(1, int((travel - 8.0) / 2.0) + 1)]
        steps = len(dxs)
        for dxv in dxs:
            dx = -dxv
            m = adsk.core.Matrix3D.create()
            m.setToRotation(kh * dx, V(1, 0, 0), adsk.core.Point3D.create(ctr.x, ctr.y, ctr.z))
            t = adsk.core.Matrix3D.create()
            t.translation = V(dx * MM, 0, 0)
            m.transformBy(t)
            moved = tbm.copy(rimB)
            tbm.transform(moved, m)
            vol = 0.0
            for ob in obst:
                x = tbm.copy(moved)
                tbm.booleanOperation(x, tbm.copy(ob), I)
                vol += x.volume * 1000.0
            if vol > worst[0]:
                worst = (vol, -dx)
        log.append('Samletest (Rim B langs skruebanen, %d trin): %s' % (steps,
                   'ingen kollision' if worst[0] < 0.05 else
                   ('kun let beroering (%.2f mm3 pr. rulle, ved %.1f mm) - OK for FDM' % (worst[0] / N, worst[1])
                    if worst[0] / N < 0.2 else 'KOLLISION %.1f mm3 ved %.1f mm' % worst)))
    except Exception as ex:
        log.append('Samletest fejlede: %s' % str(ex)[:80])

    # ---------- joints: slider + revolute linked = screw joint you can drag in Fusion ----------
    try:
        rimA_occ = [o for o in tgt.occurrences if 'Rim A' in o.component.name][0]
        rimB_occ = [o for o in tgt.occurrences if 'Rim B' in o.component.name][0]
        for o in tgt.occurrences:
            if o != rimB_occ:
                o.createForAssemblyContext(tocc).isGrounded = True
        car_occ = tgt.occurrences.addNewComponent(adsk.core.Matrix3D.create())
        car_occ.component.name = TARGET + ' Screw Carrier (hjaelpedel)'

        def org(o):
            return adsk.fusion.JointGeometry.createByPoint(o.component.originConstructionPoint.createForAssemblyContext(o))
        ji = tgt.joints.createInput(org(car_occ), org(rimA_occ))
        ji.setAsSliderJointMotion(adsk.fusion.JointDirections.XAxisJointDirection)
        j_slide = tgt.joints.add(ji)
        j_slide.name = 'Rim B slide'
        ji = tgt.joints.createInput(org(rimB_occ), org(car_occ))
        ji.setAsRevoluteJointMotion(adsk.fusion.JointDirections.XAxisJointDirection)
        j_rot = tgt.joints.add(ji)
        j_rot.name = 'Rim B turn'
        mi = tgt.motionLinks.createInput(j_slide, j_rot)
        mi.valueOne = adsk.core.ValueInput.createByString('%.4f mm' % travel)
        mi.valueTwo = adsk.core.ValueInput.createByString('%.4f deg' % turn)
        tgt.motionLinks.add(mi).name = 'Rim B screw path'
        log.append('Joints: traek i Rim B for at skrue den af/paa (%.1f mm = %.0f grader)' % (travel, abs(turn)))
    except Exception as ex:
        log.append('Joints fejlede: %s' % str(ex)[:90])
    return '\n'.join(log)


def run(context):
    ui = adsk.core.Application.get().userInterface
    try:
        ui.messageBox(build(), 'SENNEP hjulgenerator')
    except Exception:
        ui.messageBox(traceback.format_exc(), 'SENNEP hjulgenerator - fejl')
