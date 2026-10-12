"""SENNEP samlering (assembly ring) for the omni wheel.

Rim A lies face down inside the ring. Each roller rests in a cradle carved from the
roller's own shape, so it cannot tip off Rim A's short pins while Rim B is screwed on.
Reusable: the ring only exists outside r = RING_IN (> rim radius 49.6) and on the
Rim A side of the rim joint (x >= X_TOP), and the cradles are the rollers swept along
-X (the lift direction), so Rim B clears it and the closed wheel lifts straight out.

Run via MCP: exec this file, then build_ring(). Coordinates are "11 Wheel Lab" native:
wheel axis = X, Rim A outer face (bench) at x = +32.04, rim joint at x = 0.
"""
import adsk.core, adsk.fusion, math

RING_IN = 50.0      # mm, inner radius (Rim A/B max radius 49.6)
RING_OUT = 56.0     # mm, outer radius of the cradle zone
FOOT_OUT = 53.0     # mm, outer radius of the foot below the rollers
X_TOP = 0.5         # mm, ring top (just on the Rim A side of the joint)
CLEAR = 0.3         # mm, roller moved radially out by this before carving
STEP = 0.25         # mm, sweep step along -X
VARIANTS = (('core', 'Core', 'uden sleeves'), ('sleeve', 'Sleeve', 'med TPU-sleeves'))
NAME = '11 Wheel Lab Assembly Ring (samlering)'

MM = 0.1
V = lambda x, y, z: adsk.core.Vector3D.create(x * MM, y * MM, z * MM)
P = lambda x, y, z: adsk.core.Point3D.create(x * MM, y * MM, z * MM)
BT = adsk.fusion.BooleanTypes


def build_ring():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    tbm = adsk.fusion.TemporaryBRepManager.get()
    lab = [o for o in des.rootComponent.occurrences
           if o.component.name.startswith('11 Wheel Lab') and o.childOccurrences.count > 0][0].component
    comp = {o.component.name: o.component for o in lab.occurrences}
    rollers = comp['11 Wheel Lab Rollers (assembled)']
    rim_a = comp['11 Wheel Lab Rim A (motor side)'].bRepBodies.item(0)
    x_bench = rim_a.boundingBox.maxPoint.x / MM
    log = []

    def cyl(x0, x1, r0, r1=None):
        return tbm.createCylinderOrCone(P(x0, 0, 0), r0 * MM, P(x1, 0, 0), (r1 if r1 else r0) * MM)

    def blank():
        # foot r<=FOOT_OUT from the bench, 45 deg cone, cradle zone r<=RING_OUT at the top
        x_cone = 22.0
        b = cyl(X_TOP, x_cone - (RING_OUT - FOOT_OUT), RING_OUT)
        for t in (cyl(x_cone - (RING_OUT - FOOT_OUT), x_cone, RING_OUT, FOOT_OUT), cyl(x_cone, x_bench, FOOT_OUT)):
            tbm.booleanOperation(b, t, BT.UnionBooleanType)
        tbm.booleanOperation(b, cyl(X_TOP - 1, x_bench + 1, RING_IN), BT.DifferenceBooleanType)
        return b

    bodies = {}
    for key, word, label in VARIANTS:
        ring = blank()
        n = 0
        while rollers.bRepBodies.itemByName('Roller %d %s' % (n + 1, word)):
            n += 1
        # one cutter from roller 1, rotated about the wheel axis for the others
        body = rollers.bRepBodies.itemByName('Roller 1 %s' % word)
        piece = tbm.copy(body)
        if word != 'Core':  # the sleeve is a hollow tube: carve with sleeve + core
            tbm.booleanOperation(piece, tbm.copy(rollers.bRepBodies.itemByName('Roller 1 Core')), BT.UnionBooleanType)
        c = rollers.bRepBodies.itemByName('Roller 1 Core').physicalProperties.centerOfMass
        rr = math.hypot(c.y, c.z)
        m = adsk.core.Matrix3D.create()
        m.translation = adsk.core.Vector3D.create(0, c.y / rr * CLEAR * MM, c.z / rr * CLEAR * MM)
        moved = tbm.copy(piece)
        tbm.transform(moved, m)
        # keep the roller at rest too: a pure radial shift uncovers its inner flank near the ends
        tbm.booleanOperation(piece, moved, BT.UnionBooleanType)
        zone = cyl(-30, x_bench + 1, RING_OUT + 5)
        tbm.booleanOperation(zone, cyl(-31, x_bench + 2, RING_IN - 1), BT.DifferenceBooleanType)
        tbm.booleanOperation(piece, zone, BT.IntersectionBooleanType)
        # sweep along -X by doubling: after k steps the tool covers shifts 0 .. (2^k - 1) * STEP
        reach = piece.boundingBox.maxPoint.x / MM - X_TOP + 1.0
        tool, s, covered = piece, STEP, 0.0
        while covered < reach:
            cp = tbm.copy(tool)
            t = adsk.core.Matrix3D.create()
            t.translation = V(-s, 0, 0)
            tbm.transform(cp, t)
            tbm.booleanOperation(tool, cp, BT.UnionBooleanType)
            covered += s
            s *= 2
        c1 = math.atan2(c.z, c.y)
        for i in range(1, n + 1):
            ci = rollers.bRepBodies.itemByName('Roller %d Core' % i).physicalProperties.centerOfMass
            cp = tbm.copy(tool)
            r = adsk.core.Matrix3D.create()
            r.setToRotation(math.atan2(ci.z, ci.y) - c1, adsk.core.Vector3D.create(1, 0, 0), adsk.core.Point3D.create(0, 0, 0))
            tbm.transform(cp, r)
            tbm.booleanOperation(ring, cp, BT.DifferenceBooleanType)
        bodies[key] = ring

        # checks: true rollers vs ring at rest and while lifting the wheel out (-X)
        worst = 0.0
        for d in [0.0, 0.5, 1, 2, 4, 8, 12, 16, 20]:
            for i in range(1, n + 1):
                cp = tbm.copy(rollers.bRepBodies.itemByName('Roller %d %s' % (i, word)))
                tbm.booleanOperation(cp, tbm.copy(rollers.bRepBodies.itemByName('Roller %d Core' % i)), BT.UnionBooleanType)
                t = adsk.core.Matrix3D.create()
                t.translation = V(-d, 0, 0)
                tbm.transform(cp, t)
                tbm.booleanOperation(cp, tbm.copy(ring), BT.IntersectionBooleanType)
                worst = max(worst, cp.volume * 1000)
        ra = tbm.copy(rim_a)
        tbm.booleanOperation(ra, tbm.copy(ring), BT.IntersectionBooleanType)
        log.append('%s (%s): %.1f cm3, %d ruller, kollision ved loeft %.3f mm3, Rim A %.3f mm3'
                   % (key, label, ring.volume, n, worst, ra.volume * 1000))

    # replace old component and store bodies
    for o in list(lab.occurrences):
        if o.component.name == NAME:
            o.deleteMe()
    occ = lab.occurrences.addNewComponent(adsk.core.Matrix3D.create())
    occ.component.name = NAME
    bf = occ.component.features.baseFeatures.add()
    bf.startEdit()
    for key, word, label in VARIANTS:
        occ.component.bRepBodies.add(bodies[key], bf)
    bf.finishEdit()
    # names given inside the base-feature edit are lost, so name afterwards (same order)
    for i, (key, word, label) in enumerate(VARIANTS):
        occ.component.bRepBodies.item(i).name = 'Samlering %s (print bunden ned)' % label
    occ.component.bRepBodies.item(1).isLightBulbOn = False
    return '\n'.join(log)
