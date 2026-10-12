"""SENNEP V2 chassis generator.

Two builds (run them in separate MCP calls - one build per call):
- build_pod():     "21 Pod Lab" = ONE corner pod wrapped round the generated wheel in "11 Wheel Lab"
                   (pod + motor collar + bearing keeper + display copies of the motor and bearing).
- build_chassis(): "20 Chassis" = the rolling skeleton in robot coordinates: 4 pods, floor trays,
                   end trays (front/rear) + display copies of wheels, motors and bearings.

Rules (see ai-instructions/HANDOFF_CHASSIS.md):
- never touches Wheel_*, Axle_*, Spacer_*, Motor_* parameters or the wheel components (reads them).
- own parameters: Chassis_*, Brg_*, Mot_* (created on first run, edit in Change Parameters).
- pod frame = wheel frame: X = wheel axis (+X towards the motor), wheel centre at origin, Z up.
  The pod prints standing on its motor end: build direction is -X, "down" in print is +X.
- trays print flat (bottom on the plate): "down" in print is -Z.
- motor/bearing/wheel are COPIED as display bodies (no nested occurrences: their transform
  overrides get reset whenever a parameter changes).

Robot frame: origin = middle of the footprint on the floor, front = -Y, left = -X, up = +Z.
"""
import adsk.core
import adsk.fusion
import math
import traceback

POD_LAB = '21 Pod Lab'
CHASSIS = '20 Chassis'
WHEEL = '11 Wheel Lab'
MOTOR_COMP = 'Motor_1 (1)'
BEARING_COMP = 'Pillow_Block_Bearing (1)'

PARAMS = {
    'Chassis_SideGap':     (2.0, 'mm', 'Akselhovedets yderside til sidelinjen'),
    'Chassis_WheelClear':  (2.0, 'mm', 'Frigang hjul - hjulkasse'),
    'Chassis_Wall':        (4.0, 'mm', 'Vaegtykkelse pod (hjulkassens bue)'),
    'Chassis_MotorClear':  (0.1, 'mm', 'Frigang motor i roeret (pr. side) - som rillerne i hjulet: glider uden slaer'),
    'Chassis_PocketClear': (0.15, 'mm', 'Frigang lejeflange i lommen (pr. side)'),
    'Chassis_BoltAngle':   (0.0, 'deg', 'Lejets boltlinje drejet om akslen (0 = vandret, giver mest frihoejde)'),
    'Chassis_KeeperT':     (6.0, 'mm', 'Tykkelse af lejeholderen (presser flangen ned i lommen)'),
    'Chassis_InsertDia':   (4.0, 'mm', 'Hul til M3 gevindindsats'),
    'Chassis_InsertDepth': (6.0, 'mm', 'Dybde M3 gevindindsats'),
    'Chassis_CollarT':     (4.0, 'mm', 'Motorkrave tykkelse'),
    'Chassis_LugW':        (8.0, 'mm', 'Krave-tap bredde (tager drejningsmomentet)'),
    'Chassis_LugH':        (6.0, 'mm', 'Krave-tap hoejde (radialt)'),
    'Chassis_LugClear':    (0.1, 'mm', 'Frigang tap i kanal (pr. side) - lille = ingen slaer'),
    'Chassis_ToolDia':     (12.0, 'mm', 'Hul i bunden til skruetraekker op til koblingens pinolskrue'),
    'Chassis_KeeperToolDia': (10.0, 'mm', 'Vaerktoejskanal bagfra til lejeholderens skruer (skruetraekker max ~9)'),
    'Chassis_LugAngle':    (90.0, 'deg', 'Krave-tappernes vinkel om akslen (90 = op/ned, fri af lejeskruernes kanaler)'),
    'Chassis_MotorGapMin': (30.0, 'mm', 'Mindste luft mellem venstre og hoejre motor (terminaler)'),
    'Chassis_GroundMin':   (20.0, 'mm', 'Advarsel hvis frihoejden bliver mindre'),
    'Chassis_TrayH':       (40.0, 'mm', 'Bundbakkernes hoejde (vaeggene er bjaelker)'),
    'Chassis_TrayWall':    (3.0, 'mm', 'Bundbakkernes vaegtykkelse'),
    'Chassis_TrayFloor':   (3.0, 'mm', 'Bundbakkernes bundtykkelse'),
    'Chassis_PartGap':     (0.1, 'mm', 'Luft mellem to dele der boltes sammen (pr. side)'),
    'Chassis_BoltPitch':   (50.0, 'mm', 'Afstand mellem M3-bolte i samlingerne'),
    'Chassis_GridPitch':   (40.0, 'mm', 'Gitter af M3-indsatser i bunden til elektronik'),
    # KFL003 (forhandlerdata: 71 lang, 56 mellem boltene; resten maalt i Fusion-modellen)
    'Brg_Length':          (71.0, 'mm', 'KFL003 flangens laengde'),
    'Brg_HoleSpacing':     (56.0, 'mm', 'KFL003 afstand mellem boltehullerne'),
    'Brg_Hole':            (7.0, 'mm', 'KFL003 boltehul'),
    'Brg_EarW':            (23.0, 'mm', 'KFL003 oerets bredde ved huset'),
    'Brg_HubDia':          (46.0, 'mm', 'KFL003 rundt hus'),
    'Brg_FlangeT':         (7.0, 'mm', 'KFL003 flangetykkelse (bagside til oerets forside)'),
    'Brg_RingBack':        (2.0, 'mm', 'Inderring: bagkant ligger saa langt foran flangens bagside'),
    'Brg_RingL':           (17.5, 'mm', 'Inderringens laengde (med pinolskrue-krave)'),
    # motor: datasheet RS 834-7641 where it exists, rest from the Fusion model
    'Mot_GearDia':         (42.4, 'mm', 'Gearkasse diameter (datablad 42.4; maalt 41.5)'),
    'Mot_GearL':           (46.0, 'mm', 'Gearkasse laengde fra fronten (datablad 45.9)'),
    'Mot_BossDia':         (25.0, 'mm', 'Tap paa fronten'),
    'Mot_FaceToCollar':    (2.0, 'mm', 'Front til akselkrave (tap 1 + krave 1)'),
    'Mot_HolePCD':         (35.0, 'mm', 'M4-huller i fronten, diagonalt'),
    'Mot_MaxDia':          (44.6, 'mm', 'Motorens stoerste diameter (datablad)'),
    # ladder frame (robot coordinates, |x| measured from the centre line)
    'Frame_RailIn':        (196.0, 'mm', 'Laengdebjaelke: inderside (|x|)'),
    'Frame_RailOut':       (256.0, 'mm', 'Laengdebjaelke: yderside (|x|)'),
    'Frame_H':             (50.0, 'mm', 'Bjaelkehoejde (laengdebjaelker og kofangere)'),
    'Frame_Wall':          (6.0, 'mm', 'Vaeg/endeplade i bjaelkerne'),
    'Frame_CrossY1':       (60.0, 'mm', 'Indre tvaerbjaelker ved y = +-'),
    'Frame_CrossY2':       (170.0, 'mm', 'Ydre tvaerbjaelker ved y = +-'),
    'Frame_CrossH':        (40.0, 'mm', 'Tvaerbjaelkehoejde'),
    'Frame_CrossW':        (34.0, 'mm', 'Tvaerbjaelkebredde'),
    'Frame_KeyDepth':      (3.0, 'mm', 'V-noeglens hoejde (ryg i den ene del, rille i den anden)'),
    'Frame_KeyClear':      (0.1, 'mm', 'Luft i V-noeglen'),
    # LiDAR LDROBOT STL-19P (datasheet)
    'Lidar_L':             (54.0, 'mm', 'LiDAR laengde over oererne'),
    'Lidar_Body':          (38.59, 'mm', 'LiDAR kasse'),
    'Lidar_H':             (35.0, 'mm', 'LiDAR hoejde'),
    'Lidar_RotorDia':      (35.29, 'mm', 'LiDAR rotor'),
    'Lidar_Window':        (27.2, 'mm', 'Scanningsplan over LiDARens fod (35 - 7.8)'),
    # body (shell)
    'Shell_Wall':          (3.0, 'mm', 'Skallens vaegtykkelse'),
    'Shell_Top':           (250.0, 'mm', 'Skallens flade top (LiDAR-foden sidder paa BodyTop lige over)'),
    'Shell_Corner':        (70.0, 'mm', 'Affasede hjoerner set oppefra (ToF paa skraa)'),
    'Shell_Chamfer':       (35.0, 'mm', 'Affasning mellem sider og top'),
    'Shell_FrontZ':        (140.0, 'mm', 'Hvor den skraa front starter (hoejde)'),
    'Shell_SlopeY':        (150.0, 'mm', 'Den skraa fronts dybde'),
    'Shell_ArchR':         (62.0, 'mm', 'Hjulkasseudskaering i skallen (radius)'),
    # battery: 3S6P of 18650 in the clip-together holders, measured without BMS/strips
    'Bat_L':               (122.7, 'mm', 'Batterimodul laengde (6 celler)'),
    'Bat_W':               (62.1, 'mm', 'Batterimodul bredde (3 celler)'),
    'Bat_H':               (67.8, 'mm', 'Batterimodul hoejde (celle + 2 holdere)'),
    'Bat_CellDia':         (18.4, 'mm', '18650 celle diameter'),
    'Bat_CellL':           (65.0, 'mm', '18650 celle laengde'),
    'Bat_HolderH':         (10.0, 'mm', 'Celleholderens hoejde (hver ende) - skoen'),
    'Bat_Clear':           (0.5, 'mm', 'Luft om modulet i pladens kant (pr. side)'),
    'Bat_PlateT':          (3.0, 'mm', 'Batteripladens tykkelse'),
}

COLORS = {'SENNEP Chassis': (58, 60, 66), 'SENNEP Chassis Part': (200, 150, 30), 'SENNEP Tray': (95, 100, 108),
          'SENNEP Cell': (230, 130, 170), 'SENNEP Cell Holder': (25, 25, 25), 'SENNEP Steel': (175, 178, 182),
          'SENNEP Brass': (196, 160, 70), 'SENNEP Shell': (226, 222, 210)}
MM = 0.1
BIG = 4000.0   # half-space boxes must reach every corner of the whole robot


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
            prm = up.add(name, adsk.core.ValueInput.createByString(('%g %s' % (val, unit)).strip()), unit, cmt)
        out[name] = prm.value / MM if unit == 'mm' else (math.degrees(prm.value) if unit == 'deg' else prm.value)
    for name in ('RobotWidth', 'RobotLength', 'Wheelbase', 'MaxPrint', 'BodyTop', 'Axle_ToBearing', 'Axle_BearingW',
                 'Axle_CouplingL', 'Motor_ShaftDepth'):
        out[name] = up.itemByName(name).value / MM
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
                n = lib.appearances.item(i).name
                if 'Plastic' in n and 'Matte' in n:
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


class Geo:
    """TemporaryBRep helpers, all dimensions in mm."""

    def __init__(self):
        self.tbm = adsk.fusion.TemporaryBRepManager.get()
        self.U = adsk.fusion.BooleanTypes.UnionBooleanType
        self.D = adsk.fusion.BooleanTypes.DifferenceBooleanType
        self.I = adsk.fusion.BooleanTypes.IntersectionBooleanType

    def op(self, a, b, kind):
        self.tbm.booleanOperation(a, b, kind)
        return a

    def box(self, x0, x1, y0, y1, z0, z1):
        return self.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
            Pt((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2), V(1, 0, 0), V(0, 1, 0),
            abs(x1 - x0) * MM, abs(y1 - y0) * MM, abs(z1 - z0) * MM))

    def cyl(self, x0, x1, r, y=0.0, z=0.0, r1=None):
        return self.tbm.createCylinderOrCone(Pt(x0, y, z), r * MM, Pt(x1, y, z), (r if r1 is None else r1) * MM)

    def cyl_pts(self, p0, p1, r):
        return self.tbm.createCylinderOrCone(Pt(*p0), r * MM, Pt(*p1), r * MM)

    def keep_side(self, px, py, pz, nx, ny, nz):
        """Big box filling the half space {p : (p - P) . n <= 0}."""
        n = V(nx, ny, nz)
        n.normalize()
        a = V(-n.y, n.x, 0) if abs(n.z) < 0.9 else V(1, 0, 0)
        a.normalize()
        c = Pt(px - n.x * BIG / 2, py - n.y * BIG / 2, pz - n.z * BIG / 2)
        return self.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(c, n, a, BIG * MM, BIG * MM, BIG * MM))

    def cut_by(self, body, *planes):
        for p in planes:
            self.op(body, self.keep_side(*p), self.I)
        return body

    ROUND = True   # Mikkel wants plain round holes; small round holes (<= ~18 mm) bridge fine in PETG

    def teardrop(self, c, a, t, r, l0, l1):
        """Hole along axis a (unit). Round (ROUND=True), or with a 45-degree point towards t (= up in print)."""
        if self.ROUND:
            p0 = (c[0] + a[0] * l0, c[1] + a[1] * l0, c[2] + a[2] * l0)
            p1 = (c[0] + a[0] * l1, c[1] + a[1] * l1, c[2] + a[2] * l1)
            return self.cyl_pts(p0, p1, r)
        ax, tx = V(*a), V(*t)
        w = ax.crossProduct(tx)
        p0 = (c[0] + a[0] * l0, c[1] + a[1] * l0, c[2] + a[2] * l0)
        p1 = (c[0] + a[0] * l1, c[1] + a[1] * l1, c[2] + a[2] * l1)
        hole = self.cyl_pts(p0, p1, r)
        u = V(tx.x + w.x, tx.y + w.y, tx.z + w.z)
        v = V(w.x - tx.x, w.y - tx.y, w.z - tx.z)
        u.normalize()
        v.normalize()
        m = (c[0] + a[0] * (l0 + l1) / 2, c[1] + a[1] * (l0 + l1) / 2, c[2] + a[2] * (l0 + l1) / 2)
        dia = self.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(*m), u, v, 2 * r * MM, 2 * r * MM, abs(l1 - l0) * MM))
        # only the part beyond the two tangent points (at 45 deg), so the point never sticks out sideways
        k = r / math.sqrt(2)
        self.op(dia, self.keep_side(c[0] + t[0] * k, c[1] + t[1] * k, c[2] + t[2] * k, -t[0], -t[1], -t[2]), self.I)
        return self.op(hole, dia, self.U)

    def vkey(self, c, n, axis, length, depth):
        """V-shaped key (90 deg ridge) on a face: apex 'depth' out along n from point c, running along 'axis'.
        Used as ridge (union) on one part and, a bit bigger, as groove (cut) in the other: self-centring, no play."""
        nv, av = V(*n), V(*axis)
        w = av.crossProduct(nv)
        u = V(nv.x + w.x, nv.y + w.y, nv.z + w.z)
        v = V(nv.x - w.x, nv.y - w.y, nv.z - w.z)
        u.normalize()
        v.normalize()
        s = depth * math.sqrt(2)
        b = self.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(*c), u, v, s * MM, s * MM, length * MM))
        return self.op(b, self.keep_side(c[0], c[1], c[2], -n[0], -n[1], -n[2]), self.I)

    def rot(self, body, deg, axis=(1, 0, 0)):
        m = adsk.core.Matrix3D.create()
        m.setToRotation(math.radians(deg), V(*axis), Pt(0, 0, 0))
        self.tbm.transform(body, m)
        return body

    def moved(self, body, m):
        c = self.tbm.copy(body)
        self.tbm.transform(c, m)
        return c

    def inter(self, a, b):
        x = self.tbm.copy(a)
        self.tbm.booleanOperation(x, self.tbm.copy(b), self.I)
        return x.volume * 1000.0


# ISO 4762 socket head cap screws, ISO 4032 nuts, ISO 7089 washers (Mikkel's kit: M2/M3/M4)
HW = {2: dict(D=3.8, k=2.0, s=1.5, nut_s=4.0, nut_m=1.6, w_od=5.0, w_t=0.3),
      3: dict(D=5.5, k=3.0, s=2.5, nut_s=5.5, nut_m=2.4, w_od=7.0, w_t=0.5),
      4: dict(D=7.0, k=4.0, s=3.0, nut_s=7.0, nut_m=3.2, w_od=9.0, w_t=0.8)}
KIT = [(2, 12), (2, 19), (3, 12), (3, 19), (4, 12), (4, 25)]
INSERT = dict(od=4.5, L=5.0)      # M3 brass heat-set insert (approx.)
# Printed M8x1.25 (Mikkel cuts the thread himself: die on the bolt, tap in the nut)
PM8 = dict(blank=7.85, flat=3.4, head_s=13.0, head_k=5.5, nut_s=13.0, nut_m=6.5, nut_hole=7.0,
           clear=8.8, w_od=16.0, w_t=2.0, L=26.0)


class Hardware:
    """Screws, nuts, washers, inserts as temporary bodies. Local frame: bearing face at z=0, +z = head side."""

    def __init__(self, G):
        self.G = G

    def hex_prism(self, s, z0, z1):
        G = self.G
        body = None
        for k in range(3):
            b = G.box(-s, s, -s / 2, s / 2, z0, z1)
            G.rot(b, 60 * k, (0, 0, 1))
            body = b if body is None else G.op(body, b, G.I)
        return body

    def screw(self, d, L):
        G, h = self.G, HW[d]
        b = G.tbm.createCylinderOrCone(Pt(0, 0, 0), h['D'] / 2 * MM, Pt(0, 0, h['k']), h['D'] / 2 * MM)
        G.op(b, self.hex_prism(h['s'], h['k'] * 0.4, h['k'] + 1), G.D)
        G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -L), d / 2 * MM, Pt(0, 0, 0.01), d / 2 * MM), G.U)
        return b

    def nut(self, d):
        G, h = self.G, HW[d]
        b = self.hex_prism(h['nut_s'], 0, h['nut_m'])
        return G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -1), d / 2 * MM, Pt(0, 0, h['nut_m'] + 1), d / 2 * MM), G.D)

    def washer(self, d):
        G, h = self.G, HW[d]
        b = G.tbm.createCylinderOrCone(Pt(0, 0, 0), h['w_od'] / 2 * MM, Pt(0, 0, h['w_t']), h['w_od'] / 2 * MM)
        return G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -1), (d / 2 + 0.1) * MM, Pt(0, 0, h['w_t'] + 1), (d / 2 + 0.1) * MM), G.D)

    def insert(self):
        G = self.G
        b = G.tbm.createCylinderOrCone(Pt(0, 0, -INSERT['L']), INSERT['od'] / 2 * MM, Pt(0, 0, 0), INSERT['od'] / 2 * MM)
        return G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -INSERT['L'] - 1), 1.5 * MM, Pt(0, 0, 1), 1.5 * MM), G.D)

    def pbolt(self, L=PM8['L'], flats=True):
        """Printed M8 bolt blank: hex head + round shank (die-cut thread). D-D flats so it prints lying."""
        G = self.G
        b = self.hex_prism(PM8['head_s'], 0, PM8['head_k'])
        G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -L), PM8['blank'] / 2 * MM, Pt(0, 0, 0.01), PM8['blank'] / 2 * MM), G.U)
        if flats:
            G.op(b, G.box(-20, 20, -PM8['flat'], PM8['flat'], -L - 1, PM8['head_k'] + 1), G.I)
        return b

    def pnut(self):
        G = self.G
        b = self.hex_prism(PM8['nut_s'], 0, PM8['nut_m'])
        return G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -1), PM8['nut_hole'] / 2 * MM, Pt(0, 0, PM8['nut_m'] + 1), PM8['nut_hole'] / 2 * MM), G.D)

    def pwasher(self):
        G = self.G
        b = G.tbm.createCylinderOrCone(Pt(0, 0, 0), PM8['w_od'] / 2 * MM, Pt(0, 0, PM8['w_t']), PM8['w_od'] / 2 * MM)
        return G.op(b, G.tbm.createCylinderOrCone(Pt(0, 0, -1), PM8['clear'] / 2 * MM, Pt(0, 0, PM8['w_t'] + 1), PM8['clear'] / 2 * MM), G.D)

    def pbolt_stack(self, face, up, nut_face, extra=None):
        """Printed M8: washer + bolt on the head side, washer + nut on the other side."""
        t = PM8['w_t']
        out = [('P-Washer M8', self.place(self.pwasher(), face, up, extra))]
        out.append(('P-M8x%d' % PM8['L'], self.place(self.pbolt(flats=False), (face[0] + up[0] * t, face[1] + up[1] * t, face[2] + up[2] * t), up, extra)))
        dn = (-up[0], -up[1], -up[2])
        out.append(('P-Washer M8', self.place(self.pwasher(), nut_face, dn, extra)))
        out.append(('P-Nut M8', self.place(self.pnut(), (nut_face[0] + dn[0] * t, nut_face[1] + dn[1] * t, nut_face[2] + dn[2] * t), dn, extra)))
        return out

    @staticmethod
    def frame(p, up):
        """Matrix: local origin -> p (mm), local +z -> up (unit tuple)."""
        z = V(*up)
        z.normalize()
        x = V(1, 0, 0) if abs(z.x) < 0.9 else V(0, 1, 0)
        y = z.crossProduct(x)
        y.normalize()
        x = y.crossProduct(z)
        m = adsk.core.Matrix3D.create()
        m.setWithCoordinateSystem(Pt(*p), x, y, z)
        return m

    def place(self, body, p, up, extra=None):
        m = self.frame(p, up)
        if extra is not None:
            m.transformBy(extra)
        self.G.tbm.transform(body, m)
        return body

    def bolt_stack(self, d, L, face, up, nut_face=None, washer_under_nut=True, extra=None):
        """Screw + washer on 'face' (head side, 'up' points away from the part), optional washer + nut on nut_face."""
        h = HW[d]
        out = [('Washer M%d' % d, self.place(self.washer(d), face, up, extra))]
        hp = (face[0] + up[0] * h['w_t'], face[1] + up[1] * h['w_t'], face[2] + up[2] * h['w_t'])
        out.append(('M%dx%d' % (d, L), self.place(self.screw(d, L), hp, up, extra)))
        if nut_face is not None:
            dn = (-up[0], -up[1], -up[2])
            out.append(('Washer M%d' % d, self.place(self.washer(d), nut_face, dn, extra)))
            npnt = (nut_face[0] + dn[0] * h['w_t'], nut_face[1] + dn[1] * h['w_t'], nut_face[2] + dn[2] * h['w_t'])
            out.append(('Nut M%d' % d, self.place(self.nut(d), npnt, dn, extra)))
        return out


def overhangs(body, down, bed_coord, axis, max_hole_r=9.0):
    """Faces that point more than 45 degrees downwards in print (sampled normals). Skips the bed face and
    round holes lying across the print direction up to max_hole_r (they bridge fine)."""
    bad = []
    for f in body.faces:
        g = f.geometry
        if g.objectType == adsk.core.Cylinder.classType() and g.radius / MM <= max_hole_r + 0.01 and                 abs(g.axis.x * down[0] + g.axis.y * down[1] + g.axis.z * down[2]) < 0.05:
            continue
        fb = f.boundingBox
        lo = (fb.minPoint.x, fb.minPoint.y, fb.minPoint.z)[axis] / MM
        hi = (fb.maxPoint.x, fb.maxPoint.y, fb.maxPoint.z)[axis] / MM
        if abs(lo - bed_coord) < 0.01 and abs(hi - bed_coord) < 0.01:
            continue
        ev = f.evaluator
        rng = ev.parametricRange()
        pmin, pmax = rng.minPoint, rng.maxPoint
        worst = -1.0
        for i in range(5):
            for j in range(5):
                p = adsk.core.Point2D.create(pmin.x + (pmax.x - pmin.x) * (i + 0.5) / 5, pmin.y + (pmax.y - pmin.y) * (j + 0.5) / 5)
                if not ev.isParameterOnFace(p):
                    continue
                ok, n = ev.getNormalAtParameter(p)
                if ok:
                    worst = max(worst, n.x * down[0] + n.y * down[1] + n.z * down[2])
        if worst > 0.72:
            span = sorted([(fb.maxPoint.x - fb.minPoint.x) / MM, (fb.maxPoint.y - fb.minPoint.y) / MM, (fb.maxPoint.z - fb.minPoint.z) / MM])
            if worst > 0.999 and g.objectType == adsk.core.Plane.classType() and span[2] <= 20.6:
                continue        # flat bridge across a hole of max ~20 mm: prints fine
            bad.append((f.area * 100, (fb.minPoint.x / MM, fb.minPoint.y / MM, fb.minPoint.z / MM)))
    return bad


def wheel_interface(des, Q):
    """Numbers along the wheel axis (wheel frame), read from the generated wheel and Axle_* parameters."""
    def comp_bb(name):
        c = des.allComponents.itemByName(name)
        bb = None
        for b in c.bRepBodies:
            if b.isLightBulbOn:
                if bb is None:
                    bb = b.boundingBox.copy()
                else:
                    bb.combine(b.boundingBox)
        return bb
    bA, bAx = comp_bb(WHEEL + ' Rim A (motor side)'), comp_bb(WHEEL + ' Axle')
    bR = comp_bb(WHEEL + ' Rollers (assembled)')
    I = {}
    I['x_face'] = bA.maxPoint.x / MM
    I['x_head'] = bAx.minPoint.x / MM
    I['Rw'] = max(bR.maxPoint.y, bR.maxPoint.z, bA.maxPoint.y) / MM
    I['xb0'] = I['x_face'] + Q['Axle_ToBearing']
    I['xb1'] = I['xb0'] + Q['Axle_BearingW']
    I['xc1'] = I['xb1'] + Q['Axle_CouplingL']
    I['x_set'] = I['xc1'] - Q['Motor_ShaftDepth'] / 2
    I['x_mface'] = I['xc1'] + Q['Mot_FaceToCollar']
    I['x_gear'] = I['x_mface'] + Q['Mot_GearL']
    # motor + bearing display transforms (model local -> wheel frame)
    mc = des.allComponents.itemByName(MOTOR_COMP)
    tip = max(b.boundingBox.maxPoint.x for b in mc.bRepBodies) / MM
    low = min(b.boundingBox.minPoint.x for b in mc.bRepBodies) / MM
    m = adsk.core.Matrix3D.create()
    m.setToRotation(math.pi, V(0, 0, 1), Pt(0, 0, 0))
    t = adsk.core.Matrix3D.create()
    t.translation = V((I['xc1'] + tip - Q['Motor_ShaftDepth']) * MM, 0, 0)
    m.transformBy(t)
    I['m_motor'] = m
    I['x_rear'] = I['xc1'] + tip - Q['Motor_ShaftDepth'] - low
    x_ring0 = (I['xb0'] + I['xb1']) / 2 - Q['Brg_RingL'] / 2
    I['x_back'] = x_ring0 + Q['Brg_RingL'] + Q['Brg_RingBack']
    bc = des.allComponents.itemByName(BEARING_COMP)
    back_l = min(b.boundingBox.minPoint.x for b in bc.bRepBodies) / MM
    m = adsk.core.Matrix3D.create()
    m.setToRotation(math.pi, V(0, 0, 1), Pt(0, 0, 0))
    t = adsk.core.Matrix3D.create()
    t.translation = V((I['x_back'] + back_l) * MM, 0, 0)
    m.transformBy(t)
    r = adsk.core.Matrix3D.create()
    r.setToRotation(math.radians(Q['Chassis_BoltAngle']), V(1, 0, 0), Pt(0, 0, 0))
    m.transformBy(r)
    I['m_bearing'] = m
    return I


def make_pod(G, Q, I, log):
    """Corner pod, motor collar and bearing keeper as temporary bodies in the wheel frame."""
    U, D, Ix = G.U, G.D, G.I
    x_face, x_head, Rw = I['x_face'], I['x_head'], I['Rw']
    x_set, x_mface, x_back = I['x_set'], I['x_mface'], I['x_back']
    x_ear = x_back - Q['Brg_FlangeT']
    x_keep0 = x_ear - Q['Chassis_KeeperT']
    C, W, PC = Q['Chassis_WheelClear'], Q['Chassis_Wall'], Q['Chassis_PocketClear']
    xo = x_head - Q['Chassis_SideGap']            # outer face of the pod = robot side line
    xr = x_face + C                               # end of the wheel well
    Y0 = Rw + C + W                               # arch outer radius = half width of the pod
    ang = Q['Chassis_BoltAngle']
    hl, hw, hr = Q['Brg_HoleSpacing'] / 2, Q['Brg_EarW'] / 2, Q['Brg_HubDia'] / 2
    r_gear = Q['Mot_GearDia'] / 2
    r_bore = r_gear + Q['Chassis_MotorClear']
    r_so = r_bore + W
    lug_r = r_gear + Q['Chassis_LugH']
    slot_r = lug_r + 0.5
    slot_hw = Q['Chassis_LugW'] / 2 + Q['Chassis_LugClear']
    rib_r = slot_r + 3.0
    rib_hw = slot_hw + 3.0
    k_ring = hr + 3.0
    pocket_low = max(hl * abs(math.sin(math.radians(ang))) + hw + 1.5, k_ring + 0.5)
    zb = -max(pocket_low + 3.0, r_so)
    x_coll0 = x_mface - Q['Chassis_CollarT']
    x_cone1 = x_coll0 - 4.5
    r_wide = max(Q['Mot_HolePCD'] / 2 + HW[4]['w_od'] / 2 + 0.4, r_bore)   # M4 washers + heads on the collar
    # 45 deg cone from the pocket floor out to the collar zone; starts wider than 10 mm if there is no room
    r_cone0 = max(10.0, r_wide - (x_cone1 - x_back))
    x_cone0 = x_cone1 - (r_wide - r_cone0)
    x_end = I['x_gear']
    xt0 = x_set + Q['Chassis_ToolDia'] / 2 + 3.0  # full arch until behind the screwdriver hole, then 45 deg taper
    x_slot0 = x_coll0 - 1.0
    lug_ang = Q['Chassis_LugAngle']
    x_head_seat = x_keep0 + 19.0 - HW[3]['w_t'] + 0.5   # M3x19 + washer: tip ends 0.5 mm inside the keeper insert

    # outer: a tunnel that follows the wheel (flat sides below the axle, round on top), 45-deg cone to the sleeve
    pod = G.op(G.box(xo, xt0, -Y0, Y0, zb, 0), G.cyl(xo, xt0, Y0), U)
    G.op(pod, G.cyl(xt0, xt0 + (Y0 - r_so), Y0, r1=r_so), U)
    low = G.cut_by(G.box(xt0, x_end, -Y0, Y0, zb, 0), (xt0, Y0, 0, 1, 1, 0), (xt0, -Y0, 0, 1, -1, 0))
    G.op(pod, low, U)
    sleeve = G.cyl(xt0, x_end, r_so)
    G.op(sleeve, G.box(xt0, x_end, -r_so, r_so, zb, 0), U)
    G.op(sleeve, G.rot(G.box(xt0, x_end, -rib_r, rib_r, -rib_hw, rib_hw), lug_ang), U)
    G.op(pod, sleeve, U)
    G.cut_by(pod, (0, 0, zb, 0, 0, -1))           # nothing below the pod floor: the wheel must reach the ground

    def stadium(x0, x1, half_len, half_w, ring_r):
        s = G.box(x0, x1, -half_len, half_len, -half_w, half_w)
        for sy in (-1, 1):
            G.op(s, G.cyl(x0, x1, half_w, sy * half_len, 0), U)
        return G.op(s, G.cyl(x0, x1, ring_r), U)

    cav = [G.cyl(xo - 1, xr, Rw + C)]                                                       # round wheel well
    cav.append(G.rot(stadium(xr - 0.5, x_ear, hl, hw + 1.5, k_ring + 0.5), ang))           # keeper zone
    cav.append(G.rot(stadium(x_ear - 0.01, x_back, hl, hw + PC, hr + PC), ang))           # flange, snug
    cav.append(G.cyl(x_back - 0.01, x_cone0, r_cone0))                                     # coupling
    cav.append(G.cyl(x_cone0, x_cone1, r_cone0, r1=r_wide))                                # 45 deg cone
    cav.append(G.cyl(x_cone1, x_coll0 + Q['Chassis_CollarT'] + 1.0, r_wide))               # collar + M4 heads
    cav.append(G.cyl(x_cone1, x_end + 1, r_bore))                                          # gearbox, sliding fit
    for sy in (-1, 1):                                                                     # lug channels, 45 deg roof
        s = G.box(x_slot0 - slot_hw, x_end + 1, min(sy * r_gear * 0.8, sy * slot_r), max(sy * r_gear * 0.8, sy * slot_r), -slot_hw, slot_hw)
        G.cut_by(s, (x_slot0, 0, slot_hw, -1, 0, 1), (x_slot0, 0, -slot_hw, -1, 0, -1))
        cav.append(G.rot(s, lug_ang))
    # screwdriver hole from the BOTTOM up to the coupling set screw (turn the wheel so the screw points down)
    cav.append(G.teardrop((x_set, 0, 0), (0, 0, -1), (-1, 0, 0), Q['Chassis_ToolDia'] / 2, 6.0, -zb + 1))
    # bearing keeper screws from BEHIND the flange: M3x19 + washer, head in a counterbore in the pocket floor,
    # threading into brass inserts in the keeper. A straight tool channel runs out through the motor end, so a
    # screwdriver reaches the heads from the back (also with the motor in: the channel stays outside it).
    tool_r = Q['Chassis_KeeperToolDia'] / 2
    cb_r = HW[3]['w_od'] / 2 + 0.3
    for sy in (-1, 1):
        h = G.cyl(x_back - 0.01, x_head_seat, 1.7, sy * hl, 0)
        G.op(h, G.cyl(x_head_seat, x_head_seat + HW[3]['w_t'] + HW[3]['k'] + 0.5, cb_r, sy * hl, 0), U)
        x_fun = x_head_seat + HW[3]['w_t'] + HW[3]['k'] + 0.5
        G.op(h, G.cyl(x_fun - 0.01, x_fun + (tool_r - cb_r), cb_r, sy * hl, 0, r1=tool_r), U)     # 45 deg funnel
        G.op(h, G.cyl(x_fun + (tool_r - cb_r) - 0.01, x_end + 1, tool_r, sy * hl, 0), U)
        cav.append(G.rot(h, ang))
    # M3 inserts in both flat side faces (y = +-Y0) where the trays bolt on. Low where the wall is
    # thick beside the round well, two rows in the solid bearing block.
    # V-grooves in both flat faces for the keys on the frame rail (inner face) and the bumper (outer face):
    # one vertical (locates along the robot's x), one horizontal (carries the vertical load)
    XW = Q['RobotWidth'] / 2 + xo
    kd = Q['Frame_KeyDepth'] + Q['Frame_KeyClear']
    kx = XW - (Q['Frame_RailIn'] + Q['Frame_RailOut']) / 2
    kz = zb + 15.5
    kx0 = xo - 1                                   # runs out through the outer face: no downward-facing groove end
    kx1 = min(XW - (Q['Frame_RailIn'] + 4), xt0 - 1)
    for sy in (-1, 1):
        cav.append(G.vkey((kx, sy * Y0, (zb + 2 + 30) / 2), (0, -sy, 0), (0, 0, 1), 30 - zb - 2, kd))
        cav.append(G.vkey(((kx0 + kx1) / 2, sy * Y0, kz), (0, -sy, 0), (1, 0, 0), kx1 - kx0, kd))
    ins_pos = []
    side_depth = Q['Chassis_InsertDepth'] + 7.5   # M3x12 or M3x19 both fit (solid block behind the inserts)
    zl = zb + Q['Chassis_TrayFloor'] + HW[3]['w_od'] / 2 + 0.5     # lowest bolt: washer clears the tray floor
    xc = (Q['Frame_RailIn'] + Q['Frame_RailOut']) / 2     # rail centre line (robot |x|)
    for (px, pz) in ((XW - (xc - 15), zl), (XW - (xc - 15), zb + 24), (XW - (xc + 15), zl), (XW - (xc + 15), zb + 24)):
        ins_pos.append((px, pz))
        for sy in (-1, 1):
            cav.append(G.teardrop((px, sy * Y0, pz), (0, -sy, 0), (-1, 0, 0), Q['Chassis_InsertDia'] / 2,
                                  -1, side_depth))
    for c in cav:
        G.op(pod, c, D)

    collar = G.cyl(x_coll0, x_mface, r_gear - 0.25)
    for sy in (-1, 1):
        G.op(collar, G.rot(G.box(x_coll0, x_mface, min(sy * (r_gear - 2), sy * lug_r), max(sy * (r_gear - 2), sy * lug_r),
                                 -Q['Chassis_LugW'] / 2, Q['Chassis_LugW'] / 2), lug_ang), U)
    G.op(collar, G.cyl(x_coll0 - 1, x_mface + 1, Q['Mot_BossDia'] / 2 + 0.5), D)
    for k in range(4):
        a = math.radians(45 + 90 * k)
        G.op(collar, G.cyl(x_coll0 - 1, x_mface + 1, 2.2, Q['Mot_HolePCD'] / 2 * math.cos(a), Q['Mot_HolePCD'] / 2 * math.sin(a)), D)

    keeper = stadium(x_keep0, x_ear, hl, hw, k_ring)
    G.op(keeper, G.cyl(x_keep0 - 1, x_ear + 1, hr + 0.6), D)
    for sy in (-1, 1):
        G.op(keeper, G.cyl(x_ear - 0.01, x_ear + 5.0, Q['Brg_Hole'] / 2 - 0.2, sy * hl, 0), U)
        G.op(keeper, G.cyl(x_keep0 - 1, x_ear + 6, 1.7, sy * hl, 0), D)
        G.op(keeper, G.cyl(x_keep0 - 1, x_keep0 + Q['Chassis_InsertDepth'], Q['Chassis_InsertDia'] / 2, sy * hl, 0), D)  # M3 insert
    G.rot(keeper, ang)

    P = dict(xo=xo, xr=xr, Y0=Y0, zb=zb, x_end=x_end, xt0=xt0, r_so=r_so, ins_pos=ins_pos,
             x_ear=x_ear, x_keep0=x_keep0, Rw=Rw, C=C, x_coll0=x_coll0, hl=hl, x_head_seat=x_head_seat,
             k_ring=k_ring, hw=hw, PC=PC, W=W)
    return pod, collar, keeper, P


def make_hollow(G, Q, P, pod):
    """Variant: the solid block above the bearing/coupling becomes one closed chamber with 4 mm walls.
    Its end towards the wheel (the top in print) slopes 45 deg, the cone side follows the outer cone."""
    W, Y0, xt0 = P['W'], P['Y0'], P['xt0']
    rib_top = Q['Mot_GearDia'] / 2 + Q['Chassis_LugH'] + 0.5 + 3.0   # top lug rib
    z_c = max(P['k_ring'] + 0.5, rib_top) + W                          # chamber floor (in z)
    x0 = P['xr'] + W
    r_in = Y0 - W
    x_far = xt0 + (r_in - z_c)
    ch = G.cyl(x0, xt0, r_in)
    G.op(ch, G.cyl(xt0 - 0.01, x_far, r_in, r1=z_c), G.U)            # follows the outer 45 deg cone
    G.cut_by(ch, (0, 0, z_c, 0, 0, -1))                               # keep z >= z_c
    G.cut_by(ch, (x0, 0, z_c, -1, 0, 1))                              # 45 deg roof: keep x - x0 >= z - z_c
    hol = G.tbm.copy(pod)
    G.op(hol, ch, G.D)
    return hol, ch.volume * 1000.0


def display_bodies(des, I, wheel=True):
    """Motor, bearing (and wheel) bodies in the wheel frame, for display and collision checks."""
    tbm = adsk.fusion.TemporaryBRepManager.get()
    out = []
    for (cn, m, tag) in ((MOTOR_COMP, I['m_motor'], 'Motor'), (BEARING_COMP, I['m_bearing'], 'Leje')):
        for b in des.allComponents.itemByName(cn).bRepBodies:
            if b.isLightBulbOn:
                c = tbm.copy(b)
                tbm.transform(c, m)
                out.append((tag, c))
    if wheel:
        wl = des.allComponents.itemByName(WHEEL)
        for o in wl.occurrences:
            n = o.component.name
            if not o.isLightBulbOn or 'Testprint' in n or 'Roller (print' in n or 'Screw Carrier' in n or 'Assembly Ring' in n:
                continue
            for b in o.component.bRepBodies:
                if b.isLightBulbOn:
                    out.append(('Hjul', tbm.copy(b)))
    return out


def target(des, name, m=None):
    root = des.rootComponent
    tgt = des.allComponents.itemByName(name)
    if tgt is None:
        tgt = root.occurrences.addNewComponent(m or adsk.core.Matrix3D.create()).component
        tgt.name = name
    tocc = [o for o in root.occurrences if o.component == tgt][0]
    if m is not None:
        tocc.transform2 = m
    for o in list(tgt.occurrences):
        o.deleteMe()
    for i in range(tgt.features.baseFeatures.count - 1, -1, -1):
        tgt.features.baseFeatures.item(i).deleteMe()
    for i in range(tgt.bRepBodies.count - 1, -1, -1):
        tgt.bRepBodies.item(i).deleteMe()
    return tgt, tocc


def write_bodies(des, comp, items):
    """items: list of (name, temp body, appearance name or None, visible)."""
    for ap in set(i[2] for i in items if i[2]):     # create new appearances BEFORE the bodies exist
        appearance(des, ap)
    bf = comp.features.baseFeatures.add()
    bf.startEdit()
    for (_, b, ap, _) in items:
        nb = comp.bRepBodies.add(b, bf)
        # set inside the base feature edit: then it is stored with the feature and survives recomputes
        a = appearance(des, ap) if ap else None
        if a:
            try:
                nb.appearance = a
            except Exception:
                pass
    bf.finishEdit()
    bodies = [comp.bRepBodies.item(i) for i in range(comp.bRepBodies.count)]   # fixed order before renaming
    for body, (n, _, _, _) in zip(bodies, items):
        body.name = n
    for body, (_, _, ap, vis) in zip(bodies, items):
        if ap:
            a = appearance(des, ap)
            if a:
                body.appearance = a
        if not vis:
            body.isLightBulbOn = False
    PENDING.append((comp, [i[2] for i in items]))


PENDING = []   # (component, appearance names by body index): applied again at the end of a build


def finish_appearances(des):
    """A base feature in another component resets earlier body appearances, so apply them all at the end."""
    for (comp, aps) in PENDING:
        for i, ap in enumerate(aps):
            if ap and i < comp.bRepBodies.count:
                a = appearance(des, ap)
                if a:
                    comp.bRepBodies.item(i).appearance = a
    del PENDING[:]


def build_pod():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    if not app.activeDocument.name.startswith('SENNEP V2'):
        return 'STOP: aktivt dokument er "%s", ikke SENNEP V2' % app.activeDocument.name
    Q = read_params(des)
    log = []
    G = Geo()
    I = wheel_interface(des, Q)
    pod, collar, keeper, P = make_pod(G, Q, I, log)
    wo = des.userParameters.itemByName('WheelOuterOffset')
    if wo is not None and abs(wo.value / MM + P['xo']) > 0.005:
        wo.expression = '%.2f mm' % (-P['xo'])
    wocc = [o for o in des.rootComponent.occurrences if o.component.name == WHEEL][0]
    tgt, tocc = target(des, POD_LAB, wocc.transform2)
    disp = display_bodies(des, I, wheel=False)
    items = [('Pod (print standing on the motor end)', pod, 'SENNEP Chassis', True),
             ('Motor Collar (print flat)', collar, 'SENNEP Chassis Part', True),
             ('Bearing Keeper (print flat, pins up)', keeper, 'SENNEP Chassis Part', True)]
    items += [('(visning) %s' % t, b, None, True) for (t, b) in disp]
    write_bodies(des, tgt, items)
    # hollow variant beside it (+180 mm along the wheel frame's y) to compare
    hol, saved = make_hollow(G, Q, P, pod)
    mh = adsk.core.Matrix3D.create()
    mh.translation = V(0, 180 * MM, 0)
    mh.transformBy(wocc.transform2)
    htgt, hocc = target(des, POD_LAB + ' (hul variant)', mh)
    write_bodies(des, htgt, [('Housing hul (print standing on the motor end)', hol, 'SENNEP Chassis', True)])
    log.append('Hul variant: %.0f cm3 fjernet (kammer over lejet/koblingen), housing %.0f -> %.0f cm3' %
               (saved / 1000.0, pod.volume, hol.volume))
    bad = overhangs(htgt.bRepBodies.item(0), (1, 0, 0), P['x_end'], 0)
    log.append('Hul variant overhang: %s' % ('ingen' if not bad else ', '.join('%.0f mm2 @x%.0f' % (a, p[0]) for a, p in bad[:6])))

    # checks in the wheel frame
    wheel = display_bodies(des, I, wheel=True)
    hits = []
    for (pn, pb) in (('pod', pod), ('krave', collar), ('holder', keeper)):
        for (t, ob) in wheel:
            v = G.inter(pb, ob)
            if v > 0.05:
                hits.append('%s/%s %.1f mm3' % (pn, t, v))
    v = G.inter(pod, collar) + G.inter(pod, keeper)
    if v > 0.05:
        hits.append('pod/krave+holder %.1f mm3' % v)
    log.append('Kollisioner: ' + ('ingen' if not hits else '; '.join(hits[:8])))
    env = G.cyl(P['xo'] + Q['Chassis_SideGap'] - P['C'] + 0.05, I['x_face'] + P['C'] - 0.05, P['Rw'] + P['C'] - 0.05)
    log.append('Hjulets frigangsvolumen (%.1f mm) mod pod: %.2f mm3' % (P['C'], G.inter(pod, env)))
    podB = tgt.bRepBodies.item(0)
    bb = podB.boundingBox
    dims = [(bb.maxPoint.x - bb.minPoint.x) / MM, (bb.maxPoint.y - bb.minPoint.y) / MM, (bb.maxPoint.z - bb.minPoint.z) / MM]
    log.append('Pod: %.0f (hoejde i print) x %.0f x %.0f mm, %.0f cm3 %s' % (dims[0], dims[1], dims[2], podB.volume,
               'OK' if max(dims) <= Q['MaxPrint'] else '<< STOERRE END MaxPrint'))
    bad = overhangs(podB, (1, 0, 0), P['x_end'], 0)
    log.append('Overhang i printretningen: %s' % ('ingen' if not bad else ', '.join('%.0f mm2 @x%.0f' % (a, p[0]) for a, p in bad[:6])))
    for i, nm in ((1, 'krave'), (2, 'holder')):
        b = tgt.bRepBodies.item(i)
        bad = overhangs(b, (-1, 0, 0) if i == 2 else (1, 0, 0), b.boundingBox.minPoint.x / MM if i == 2 else b.boundingBox.maxPoint.x / MM, 0)
        if bad:
            log.append('Overhang %s: %d flader' % (nm, len(bad)))
    gc = P['Rw'] + P['zb']
    log.append('Frihoejde under poden: %.1f mm. Skruetraekker: hul %.0f mm, %.0f mm dybt til pinolskruen' %
               (gc, Q['Chassis_ToolDia'], P['Y0'] - des.userParameters.itemByName('Axle_BearingDia').value / MM / 2))
    return '\n'.join(log)


def frame_numbers(des, Q, G, log):
    """Everything the frame needs, in robot coordinates (mm)."""
    I = wheel_interface(des, Q)
    pod, collar, keeper, P = make_pod(G, Q, I, log)
    wocc = [o for o in des.rootComponent.occurrences if o.component.name == WHEEL][0]
    F = dict(I=I, P=P, pod=pod, collar=collar, keeper=keeper)
    F['ZW'] = wocc.transform2.translation.z / MM
    F['W2'], F['L2'] = Q['RobotWidth'] / 2, Q['RobotLength'] / 2
    F['XW'] = F['W2'] + P['xo']
    F['YW'] = Q['Wheelbase'] / 2
    F['y_in'], F['y_out'] = F['YW'] - P['Y0'], F['YW'] + P['Y0']
    F['x_blk'] = F['XW'] - P['xt0']
    F['x_slv'] = F['XW'] - P['x_end']
    F['Zb'] = F['ZW'] + P['zb']
    F['Zr'] = F['Zb'] + Q['Frame_H']
    F['Zc0'] = F['Zr'] - Q['Frame_CrossH']
    F['xr0'], F['xr1'] = Q['Frame_RailIn'], Q['Frame_RailOut']
    F['yc'] = [-Q['Frame_CrossY2'], -Q['Frame_CrossY1'], Q['Frame_CrossY1'], Q['Frame_CrossY2']]
    F['kx'] = (F['xr0'] + F['xr1']) / 2                                 # vertical key position (|x|)
    F['rod_x'] = 60.0                                                   # through-channel for a rod (front to back)
    F['kz'] = F['ZW'] + P['zb'] + 15.5                                   # horizontal key height
    F['kxa'] = max(F['xr0'] + 4, F['x_blk'] + 1)
    F['kxb'] = F['xr1'] - 4
    # pod inserts on the flat faces, robot |x| and z
    F['ins'] = [(F['XW'] - px, F['ZW'] + pz) for (px, pz) in P['ins_pos']]

    def corner(sx, sy):
        m = adsk.core.Matrix3D.create()
        if sx > 0:
            m.setToRotation(math.pi, V(0, 0, 1), Pt(0, 0, 0))
        t = adsk.core.Matrix3D.create()
        t.translation = V(sx * F['XW'] * MM, sy * F['YW'] * MM, F['ZW'] * MM)
        m.transformBy(t)
        return m
    F['corner'] = corner
    return F


def make_frame(G, Q, F):
    """Ladder frame parts (temporary bodies, robot coordinates) + bolt list.
    All beams are an upside-down U (web on top, open below): they print web-down with no overhang,
    every bolt head/nut is reachable from below. Every joint = V-key (ridge into groove) + bolts."""
    U, D = G.U, G.D
    T, g, kd, kc = Q['Frame_Wall'], Q['Chassis_PartGap'], Q['Frame_KeyDepth'], Q['Frame_KeyClear']
    Zb, Zr, Zc0 = F['Zb'], F['Zr'], F['Zc0']
    xr0, xr1 = F['xr0'], F['xr1']
    y_in, y_out, L2, W2 = F['y_in'], F['y_out'], F['L2'], F['W2']
    cw = Q['Frame_CrossW'] / 2
    DOWN = (0, 0, -1)                     # 'up' in print for these parts (they print upside down)
    parts, bolts, grooves = [], [], []    # bolts: (kind, centre, axis pointing to the head side, length, nut?)

    def hole(c, a, l0, l1, r=1.7):
        return G.teardrop(c, a, DOWN, r, l0, l1)
    RB = PM8['clear'] / 2                 # printed M8 clearance hole radius
    KB = 'P-M8'

    def rows(z0, z1, edge=8.0):
        """Two bolt rows centred on the plate height between z0 and the underside of the web z1."""
        m = (z0 + z1) / 2
        return (m - ((z1 - z0) / 2 - edge), m + ((z1 - z0) / 2 - edge))

    def ridge(x, y, n, z0):
        """Vertical ridge from z0 up to the top face (the print bed): no downward-facing end."""
        return G.vkey((x, y, (z0 + Zr) / 2), n, (0, 0, 1), Zr - z0, kd)

    def groove(x, y, n, z0):
        """Vertical groove through the whole height (open at the bottom edge too)."""
        return G.vkey((x, y, (z0 - 1 + Zr + 1) / 2), n, (0, 0, 1), Zr - z0 + 2, kd + kc)

    def diamonds(body, axis, centres, half):
        for c in centres:
            k = 1 / math.sqrt(2)
            if axis == 'x':
                d = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
                    Pt(*c), V(0, k, k), V(0, -k, k), half * math.sqrt(2) * MM, half * math.sqrt(2) * MM, 40 * MM))
            else:
                d = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(
                    Pt(*c), V(k, 0, k), V(-k, 0, k), half * math.sqrt(2) * MM, half * math.sqrt(2) * MM, 40 * MM))
            G.op(body, d, D)

    def u_beam_y(x0, x1, y0, y1, z0, z1, close0, close1):
        """Upside-down U along y: outer box minus the hollow, optional end plates."""
        b = G.box(x0, x1, y0, y1, z0, z1)
        h = G.box(x0 + T, x1 - T, y0 + (T if close0 else -1), y1 - (T if close1 else -1), z0 - 1, z1 - T)
        return G.op(b, h, D)

    def u_beam_x(x0, x1, y0, y1, z0, z1, close0, close1):
        b = G.box(x0, x1, y0, y1, z0, z1)
        h = G.box(x0 + (T if close0 else -1), x1 - (T if close1 else -1), y0 + T, y1 - T, z0 - 1, z1 - T)
        return G.op(b, h, D)

    # ---------- side rails: 2 halves per side, joined at y = 0 ----------
    for sx in (-1, 1):
        xa, xb = sorted((sx * xr0, sx * xr1))
        xm = sx * F['kx']
        for sy in (-1, 1):
            ya, yb = sorted((sy * (y_in - g), sy * g))
            r = u_beam_y(xa, xb, ya, yb, Zb, Zr, True, True)
            # key into the pod: ridges on the end plate
            ye = sy * (y_in - g)
            G.op(r, ridge(xm, ye, (0, sy, 0), Zb + 3), U)
            xa_, xb_ = sx * F['kxa'], sx * F['kxb']
            G.op(r, G.vkey(((xa_ + xb_) / 2, ye, F['kz']), (0, sy, 0), (1, 0, 0), abs(xb_ - xa_), kd), U)
            for (ax, az) in F['ins']:
                if xr0 + 3 < ax < xr1 - 3:
                    c = (sx * ax, ye, az)
                    G.op(r, hole(c, (0, -sy, 0), -1, T + 1), D)
                    bolts.append(('M3x12 i indsats', (sx * ax, ye - sy * T, az), (0, -sy, 0), 12, False))
            # centre joint: the front half carries the ridge, the rear half the groove
            yj = sy * g
            G.op(r, ridge(xm, yj, (0, 1, 0), Zb + 3) if sy < 0 else groove(xm, yj, (0, 1, 0), Zb), U if sy < 0 else D)
            for bx in (xm - 12, xm + 12):
                for bz in rows(Zb, Zr - T):
                    G.op(r, hole((bx, yj, bz), (0, 1, 0), -T - 1, T + 1, RB), D)
                    if sy < 0:
                        bolts.append((KB, (bx, -g - T, bz), (0, -1, 0), PM8['L'], (bx, g + T, bz)))
            # cross member joints in the inner wall: groove + 2x2 bolts
            xi = sx * xr0
            for yc in F['yc']:
                if sy * yc <= 0:
                    continue
                G.op(r, groove(xi, yc, (sx, 0, 0), Zb), D)
                for bz in rows(Zc0, Zr - T):
                    G.op(r, hole((xi, yc, bz), (1, 0, 0), -T - 1, T + 1, RB), D)
                    bolts.append((KB, (xi - sx * (T + g), yc, bz), (-sx, 0, 0), PM8['L'], (xi + sx * T, yc, bz)))
            # lightening holes (diamonds) in both walls, between the cross members
            ys = [y for y in (sy * 30.0, sy * 115.0) if abs(y) + 16 < y_in - T]
            diamonds(r, 'x', [(xa + T / 2, y, (Zb + Zr - T) / 2) for y in ys] + [(xb - T / 2, y, (Zb + Zr - T) / 2) for y in ys], 13)
            parts.append(('Rail %s %s' % ('left' if sx < 0 else 'right', 'front' if sy < 0 else 'rear'), r))

    # ---------- cross members: 2 halves each, joined at x = 0 ----------
    for yc in F['yc']:
        for sx in (-1, 1):
            xa, xb = sorted((sx * g, sx * (xr0 - g)))
            c = u_beam_x(xa, xb, yc - cw, yc + cw, Zc0, Zr, True, True)
            xe = sx * (xr0 - g)
            G.op(c, ridge(xe, yc, (sx, 0, 0), Zc0 + 3), U)
            for bz in rows(Zc0, Zr - T):
                G.op(c, hole((xe, yc, bz), (1, 0, 0), -T - 1, T + 1, RB), D)
            # centre joint: left half ridge, right half groove
            xj = sx * g
            G.op(c, ridge(xj, yc, (1, 0, 0), Zc0 + 3) if sx < 0 else groove(xj, yc, (1, 0, 0), Zc0), U if sx < 0 else D)
            for bz in rows(Zc0, Zr - T):
                G.op(c, hole((xj, yc, bz), (1, 0, 0), -T - 1, T + 1, RB), D)
                if sx < 0:
                    bolts.append((KB, (-g - T, yc, bz), (-1, 0, 0), PM8['L'], (g + T, yc, bz)))
            diamonds(c, 'y', [(sx * x, yc + s * (cw - T / 2), (Zc0 + Zr - T) / 2) for x in (F['rod_x'], 130.0) for s in (-1, 1)], 10)
            # holes in the web for plates on top (battery, mast, electronics): every 30 mm
            for k in range(1, int((xr0 - 20) // 30) + 1):
                G.op(c, G.tbm.createCylinderOrCone(Pt(sx * 30 * k - sx * 15, yc, Zr - T - 1), RB * MM, Pt(sx * 30 * k - sx * 15, yc, Zr + 1), RB * MM), D)
            parts.append(('Cross y%+.0f %s' % (yc, 'left' if sx < 0 else 'right'), c))

    # ---------- bumpers front/rear: 3 segments, bolted to the pods' outer faces ----------
    xs3 = W2 / 3
    segs = [(-W2, -xs3), (-xs3, xs3), (xs3, W2)]
    for sy in (-1, 1):
        ya, yb = sorted((sy * (y_out + g), sy * L2))
        for k, (x0, x1) in enumerate(segs):
            xa, xb = x0 + (g if k > 0 else 0), x1 - (g if k < 2 else 0)
            b = G.box(xa, xb, ya, yb, Zb, Zr)
            G.op(b, G.box(xa + T, xb - T, ya + T, yb - T, Zb - 1, Zr - T), D)
            yw = sy * (y_out + g)
            if k != 1:
                sx = -1 if k == 0 else 1
                G.op(b, ridge(sx * F['kx'], yw, (0, -sy, 0), Zb + 3), U)
                G.op(b, G.vkey((sx * (F['kxa'] + F['kxb']) / 2, yw, F['kz']), (0, -sy, 0), (1, 0, 0), F['kxb'] - F['kxa'], kd), U)
                for (ax, az) in F['ins']:
                    c = (sx * ax, yw, az)
                    G.op(b, hole(c, (0, sy, 0), -1, T + 1), D)
                    bolts.append(('M3x12 i indsats', (sx * ax, yw + sy * T, az), (0, sy, 0), 12, False))
            # segment joints at x = +-xs3: middle segment has the grooves
            for (xj, sxj) in ((x0, -1), (x1, 1)):
                if abs(abs(xj) - xs3) > 0.01:
                    continue
                xjf = xj + (g if xj == x0 else -g)
                ym = (ya + yb) / 2
                nj = (sxj, 0, 0) if k != 1 else (-sxj, 0, 0)
                G.op(b, groove(xjf, ym, nj, Zb) if k == 1 else ridge(xjf, ym, nj, Zb + 3), D if k == 1 else U)
                for by in (ym - 13, ym + 13):
                    for bz in rows(Zb, Zr - T):
                        G.op(b, hole((xj, by, bz), (1, 0, 0), -T - 1, T + 1, RB), D)
                        if k == 1:
                            bolts.append((KB, (xj - sxj * (T + g), by, bz), (-sxj, 0, 0), PM8['L'], (xj + sxj * (T + g), by, bz)))
            # diamonds in line with the cross members' (x = +-rod_x): a rod can go through the whole robot
            zd = (Zc0 + Zr - T) / 2
            cs = [(x, y, zd) for x in (-F['rod_x'], F['rod_x']) if xa + 12 < x < xb - 12
                  for y in (ya + T / 2, yb - T / 2)]
            if cs:
                diamonds(b, 'y', cs, 10)
            parts.append(('Bumper %s %s' % ('front' if sy < 0 else 'rear', ('left', 'middle', 'right')[k]), b))
    return parts, bolts


def make_lidar(G, Q, F):
    """LiDAR mast (printed standing) on the two inner cross members + a model of the STL-19P on top."""
    U, D = G.U, G.D
    Zr, y1 = F['Zr'], Q['Frame_CrossY1']
    zt = Q['BodyTop']                        # LiDAR foot = top of the body
    pt = 4.0
    base = G.box(-25, 25, -y1 - 12, y1 + 12, Zr, Zr + pt)
    tube = G.box(-15, 15, -15, 15, Zr, zt - 12 - pt)
    flare = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(0, 0, zt - pt - 6), V(1, 0, 0), V(0, 1, 0), 54 * MM, 48 * MM, 12 * MM))
    for (nx, ny, px, py) in ((1, 0, 15, 0), (-1, 0, -15, 0), (0, 1, 0, 15), (0, -1, 0, -15)):
        G.cut_by(flare, (px, py, zt - pt - 12, nx, ny, -1))
    top = G.box(-27, 27, -24, 24, zt - pt, zt)
    mast = G.op(G.op(G.op(base, tube, U), flare, U), top, U)
    za = zt - pt - 14                                  # apex of the hollow's pyramid roof
    hol = G.box(-12, 12, -12, 12, Zr + pt, za)
    for (nx, ny) in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        G.cut_by(hol, (nx * 12, ny * 12, za - 12, nx, ny, 1))
    G.op(mast, hol, D)
    # LiDAR holes (datasheet: 3x d2.5, two 46.8 apart, the third 31.92 lower) -> M2 + nut
    holes = [(-23.4, 8.8), (23.4, 8.8), (0.0, -23.1)]
    for (hx, hy) in holes:
        G.op(mast, G.tbm.createCylinderOrCone(Pt(hx, hy, zt - pt - 1), 1.3 * MM, Pt(hx, hy, zt + 1), 1.3 * MM), D)
        # M2 nut + washer from below: 6 mm hole ending in a 45 deg cone (no flat ceiling)
        G.op(mast, G.tbm.createCylinderOrCone(Pt(hx, hy, zt - pt - 13), 3.0 * MM, Pt(hx, hy, zt - pt - 1.7), 3.0 * MM), D)
        G.op(mast, G.tbm.createCylinderOrCone(Pt(hx, hy, zt - pt - 1.71), 3.0 * MM, Pt(hx, hy, zt - pt), 1.3 * MM), D)
    # bolts into the cross member webs
    mb = []
    for yy in (-y1, y1):
        for xx in (-15.0, 15.0):
            G.op(mast, G.tbm.createCylinderOrCone(Pt(xx, yy, Zr - 1), PM8['clear'] / 2 * MM, Pt(xx, yy, Zr + pt + 1), PM8['clear'] / 2 * MM), D)
            mb.append(('P-M8', (xx, yy, Zr + pt), (0, 0, 1), PM8['L'], (xx, yy, Zr - Q['Frame_Wall'])))
    # LiDAR model
    lb = Q['Lidar_Body']
    li = G.box(-lb / 2, lb / 2, -lb / 2, lb / 2, zt, zt + Q['Lidar_H'] - 12.8)
    G.op(li, G.tbm.createCylinderOrCone(Pt(0, 0, zt + Q['Lidar_H'] - 12.8 - 0.01), Q['Lidar_RotorDia'] / 2 * MM,
                                        Pt(0, 0, zt + Q['Lidar_H']), Q['Lidar_RotorDia'] / 2 * MM), U)
    for (hx, hy) in holes:
        ear = G.tbm.createCylinderOrCone(Pt(hx, hy, zt), 3.5 * MM, Pt(hx, hy, zt + 3), 3.5 * MM)
        G.op(li, ear, U)
        G.op(li, G.tbm.createCylinderOrCone(Pt(hx, hy, zt - 1), 1.25 * MM, Pt(hx, hy, zt + 4), 1.25 * MM), D)
    scan = zt + Q['Lidar_Window']
    return mast, li, mb, scan


def build_chassis():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    if not app.activeDocument.name.startswith('SENNEP V2'):
        return 'STOP: aktivt dokument er "%s", ikke SENNEP V2' % app.activeDocument.name
    root = des.rootComponent
    Q = read_params(des)
    log = []
    G = Geo()
    F = frame_numbers(des, Q, G, log)
    pod, collar, keeper, P, I = F['pod'], F['collar'], F['keeper'], F['P'], F['I']
    parts, bolts = make_frame(G, Q, F)
    mast, lidar, mbolts, scan = make_lidar(G, Q, F)
    bolts += mbolts
    disp = display_bodies(des, I, wheel=True)
    corners = [(-1, -1, 'FL'), (1, -1, 'FR'), (-1, 1, 'RL'), (1, 1, 'RR')]

    for o in root.occurrences:
        if o.component.name == '10 Drive Module':
            o.isLightBulbOn = False
    tgt, tocc = target(des, CHASSIS, adsk.core.Matrix3D.create())
    items = []
    for (sx, sy, n) in corners:
        m = F['corner'](sx, sy)
        items.append(('Pod %s' % n, G.moved(pod, m), 'SENNEP Chassis', True))
        items.append(('Collar %s' % n, G.moved(collar, m), 'SENNEP Chassis Part', True))
        items.append(('Keeper %s' % n, G.moved(keeper, m), 'SENNEP Chassis Part', True))
    for (n, b) in parts:
        items.append((n, b, 'SENNEP Tray', True))
    items.append(('LiDAR Mast (print standing)', mast, 'SENNEP Chassis Part', True))
    n_print = len(items)
    items.append(('(visning) LiDAR STL-19P', lidar, 'SENNEP Cell Holder', True))
    for (sx, sy, n) in corners:
        m = F['corner'](sx, sy)
        for (t, b) in disp:
            items.append(('(visning) %s %s' % (t, n), G.moved(b, m), None, True))
    write_bodies(des, tgt, items)

    # ---------- checks ----------
    printed = [(items[i][0], items[i][1]) for i in range(n_print)]
    others = [(items[i][0], items[i][1]) for i in range(n_print, len(items))]
    hits = []
    for i in range(len(printed)):
        for j in range(i + 1, len(printed)):
            a, b = printed[i][1], printed[j][1]
            if a.boundingBox.intersects(b.boundingBox):
                v = G.inter(a, b)
                if v > 0.05:
                    hits.append('%s/%s %.1f' % (printed[i][0], printed[j][0], v))
        for (on, ob) in others:
            if printed[i][1].boundingBox.intersects(ob.boundingBox):
                v = G.inter(printed[i][1], ob)
                if v > 0.05:
                    hits.append('%s/%s %.1f' % (printed[i][0], on, v))
    log.append('Kollisioner (mm3): ' + ('ingen' if not hits else '; '.join(hits[:12]) + (' (+%d)' % (len(hits) - 12) if len(hits) > 12 else '')))
    mp = Q['MaxPrint']
    big, diag = [], []
    for (n, b) in printed:
        if n.startswith(('Pod', 'Collar', 'Keeper')):
            continue
        bb = b.boundingBox
        d = sorted([(bb.maxPoint.x - bb.minPoint.x) / MM, (bb.maxPoint.y - bb.minPoint.y) / MM, (bb.maxPoint.z - bb.minPoint.z) / MM])
        if d[2] > mp and (d[1] + d[2]) / math.sqrt(2) <= mp:
            diag.append('%s %.0f' % (n, d[2]))
        elif d[2] > mp:
            big.append('%s %.0f' % (n, d[2]))
    log.append('Dele paa pladen (%.0f x %.0f): %s%s' % (mp, mp, 'alle passer' if not big else 'PASSER IKKE: ' + ', '.join(big),
               ('; diagonalt: ' + ', '.join(diag)) if diag else ''))
    low = [n for (n, b) in printed if b.boundingBox.minPoint.z / MM < F['Zb'] - 0.01]
    log.append('Under bunden (z < %.1f): %s' % (F['Zb'], 'intet' if not low else 'FEJL: ' + ', '.join(low)))
    # overhang: beams print upside down (top on the plate), the mast standing
    oh = []
    seen = set()
    for body in tgt.bRepBodies:
        kind = body.name.split(' ')[0]
        if kind in ('Rail', 'Cross', 'Bumper') and (kind, 'middle' in body.name) not in seen:
            seen.add((kind, 'middle' in body.name))
            bad = overhangs(body, (0, 0, 1), F['Zr'], 2)
            if bad:
                oh.append('%s: %d (fx %.0f mm2 @ %.0f,%.0f,%.0f)' % (body.name, len(bad), bad[0][0], bad[0][1][0], bad[0][1][1], bad[0][1][2]))
        if body.name.startswith('LiDAR Mast'):
            bad = overhangs(body, (0, 0, -1), F['Zr'], 2)
            if bad:
                oh.append('mast: %d (fx %.0f mm2 @ %.0f,%.0f,%.0f)' % (len(bad), bad[0][0], bad[0][1][0], bad[0][1][1], bad[0][1][2]))
    log.append('Overhang: ' + ('ingen' if not oh else '; '.join(oh)))
    # LiDAR: worst-case 2 deg pitch over the farthest corner
    rmax = math.hypot(F['W2'], F['L2'])
    drop = rmax * math.tan(math.radians(2))
    log.append('LiDAR: fod z=%.0f, scanningsplan z=%.1f, top z=%.1f. Ved 2 grader haeldning falder planet %.0f mm ved hjoernerne -> skallen skal vaere under z=%.0f' %
               (Q['BodyTop'], scan, Q['BodyTop'] + Q['Lidar_H'], drop, scan - drop - 3))
    # through-channel for a rod at x = +-rod_x (diamond 9 mm half-diagonal), front to back
    k = 1 / math.sqrt(2)
    zd = (F['Zc0'] + F['Zr'] - Q['Frame_Wall']) / 2
    rod_hits = []
    for sxr in (-1, 1):
        rod = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(sxr * F['rod_x'], 0, zd), V(k, 0, k), V(-k, 0, k),
                                                                      9 * math.sqrt(2) * MM, 9 * math.sqrt(2) * MM, (2 * F['L2'] + 2) * MM))
        for (n, b) in printed + others:
            if rod.boundingBox.intersects(b.boundingBox) and G.inter(rod, b) > 0.05:
                rod_hits.append(n)
    log.append('Gennemgaaende kanal (diamant, x = +-%.0f, z = %.0f) fra front til bag: %s' %
               (F['rod_x'], zd, 'fri' if not rod_hits else 'BLOKERET af ' + ', '.join(sorted(set(rod_hits)))))
    # BOM
    from collections import Counter
    cnt = Counter(k for (k, _, _, _, _) in bolts)
    npod = cnt['M3x12 i indsats']
    npm8 = cnt['P-M8'] + 8                      # + 8 for the battery plate (build_battery)
    log.append('Koebt hardware: M3 (12 eller 19 mm) %d til hjoernerne + 8 M3x19 til lejeholderne = %d M3 (saet: 20+20), '
               'M3-skiver %d, M3-indsatser %d, M4x12 16 + M4-skiver 16 til motorkraverne' % (npod, npod + 8, npod + 8, npod + 8))
    log.append('Printet hardware: %d P-M8x%d bolte, %d P-M8 moetrikker, %d P-skiver (du skaerer gevind M8x1.25)' %
               (npm8, PM8['L'], npm8, 2 * npm8))
    log.append('Dele: %d bjaelker/kofangere + mast + 4 hjoerner' % len(parts))

    # ---------- hardware examples: every bolt of the front-left quarter ----------
    HWm = Hardware(G)
    ex = []
    for (kind, face, up, L, nutf) in bolts:
        if face[0] > 5 or face[1] > 5:
            continue
        if kind == 'P-M8':
            ex += HWm.pbolt_stack(face, up, nutf)
            continue
        ex += HWm.bolt_stack(3, L, face, up, nut_face=nutf if nutf else None)
        if not nutf:
            ex.append(('Insert M3', HWm.place(HWm.insert(), (face[0] - up[0] * Q['Frame_Wall'], face[1] - up[1] * Q['Frame_Wall'], face[2] - up[2] * Q['Frame_Wall']), up)))
    m = F['corner'](-1, -1)
    for sy in (-1, 1):
        ex += HWm.bolt_stack(3, 19, (P['x_keep0'], sy * P['hl'], 0), (-1, 0, 0), extra=m)
        ex.append(('Insert M3', HWm.place(HWm.insert(), (I['x_back'], sy * P['hl'], 0), (-1, 0, 0), m)))
    for k in range(4):
        a = math.radians(45 + 90 * k)
        ex += HWm.bolt_stack(4, 12, (P['x_coll0'], Q['Mot_HolePCD'] / 2 * math.cos(a), Q['Mot_HolePCD'] / 2 * math.sin(a)), (-1, 0, 0), extra=m)
    hc, _ = target(des, '91 Hardware (placed)', adsk.core.Matrix3D.create())
    write_bodies(des, hc, [(n, b, 'SENNEP Brass' if n.startswith('Insert') else ('SENNEP Chassis Part' if n.startswith('P-') else 'SENNEP Steel'), True)
                           for (n, b) in ex])
    hits = []
    for (hn, hb) in ex:
        if hn.startswith('Insert'):
            continue
        for (n, b) in printed:
            if hb.boundingBox.intersects(b.boundingBox) and G.inter(hb, b) > 0.05:
                hits.append('%s/%s' % (hn, n))
    log.append('Bolte (forreste venstre kvart, %d dele) mod plast: %s' % (len(ex), 'fri' if not hits else ', '.join(sorted(set(hits))[:8])))
    finish_appearances(des)
    return '\n'.join(log)


def build_hardware_library():
    """'90 Hardware': every size from Mikkel's kit + M3 insert, in a row beside the robot."""
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    G = Geo()
    HWm = Hardware(G)
    items = []
    y = 0.0
    for (d, L) in KIT:
        items.append(('M%dx%d (unbrako)' % (d, L), HWm.place(HWm.screw(d, L), (-700, y, 40), (0, 0, 1)), None, True))
        y += 15
    for d in (2, 3, 4):
        items.append(('Nut M%d' % d, HWm.place(HWm.nut(d), (-700, y, 0), (0, 0, 1)), None, True))
        items.append(('Washer M%d' % d, HWm.place(HWm.washer(d), (-680, y, 0), (0, 0, 1)), None, True))
        y += 15
    items.append(('Insert M3 (messing)', HWm.place(HWm.insert(), (-700, y, 5), (0, 0, 1)), None, True))
    y += 25
    items.append(('P-M8x%d printet bolt (D-D, print liggende)' % PM8['L'], HWm.place(HWm.pbolt(), (-700, y, 40), (0, 0, 1)), None, True))
    items.append(('P-M8 printet moetrik (Oe7.0 -> tap M8x1.25)', HWm.place(HWm.pnut(), (-670, y, 0), (0, 0, 1)), None, True))
    items.append(('P-M8 printet skive', HWm.place(HWm.pwasher(), (-645, y, 0), (0, 0, 1)), None, True))
    hc, _ = target(des, '90 Hardware', adsk.core.Matrix3D.create())
    write_bodies(des, hc, [(n, b, 'SENNEP Brass' if 'Insert' in n else ('SENNEP Chassis Part' if n.startswith('P-') else 'SENNEP Steel'), v)
                           for (n, b, _, v) in items])
    finish_appearances(des)
    return '90 Hardware: %d dele' % len(items)


def build_battery():
    """'31 Battery Module' (cells + clip holders) and '32 Battery Mount' (plate + 2 clamp bridges + screws),
    in the front middle tray, screwed into the tray's insert grid."""
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    if not app.activeDocument.name.startswith('SENNEP V2'):
        return 'STOP: aktivt dokument er "%s", ikke SENNEP V2' % app.activeDocument.name
    root = des.rootComponent
    Q = read_params(des)
    log = []
    G = Geo()
    U, D = G.U, G.D
    HWm = Hardware(G)
    F = frame_numbers(des, Q, G, log)
    Y1, Y2, Tw = Q['Frame_CrossY1'], Q['Frame_CrossY2'], Q['Frame_Wall']
    pitch = Q['Chassis_GridPitch']
    cx, cy = 0.0, -(Y1 + Y2) / 2              # front bay, between the two front cross members
    z_boss = F['Zr']                          # the plate lies on the cross members
    T = Q['Bat_PlateT']
    z_mod = z_boss + T                        # module stands on the plate
    L, W, H = Q['Bat_L'], Q['Bat_W'], Q['Bat_H']
    px, py = L / 6, W / 3
    HT, lip = Q['Bat_HolderH'], (H - Q['Bat_CellL']) / 2
    z_top = z_mod + H

    # ---------- battery module: 18 cells (3 parallel groups of 6, alternating) + 36 holder clips ----------
    mod = []
    for j in range(3):
        up = (j % 2 == 0)
        for i in range(6):
            x = cx - L / 2 + px * (i + 0.5)
            y = cy - W / 2 + py * (j + 0.5)
            z0 = z_mod + lip
            cell = G.tbm.createCylinderOrCone(Pt(x, y, z0), Q['Bat_CellDia'] / 2 * MM, Pt(x, y, z0 + Q['Bat_CellL'] - 0.6), Q['Bat_CellDia'] / 2 * MM)
            zb_ = z0 + Q['Bat_CellL'] - 0.6 if up else z0
            nub = G.tbm.createCylinderOrCone(Pt(x, y, zb_), 3.5 * MM, Pt(x, y, zb_ + (0.6 if up else -0.0)), 3.5 * MM) if up else None
            if nub is not None:
                G.op(cell, nub, U)
            mod.append(('Cell %d-%d %s' % (j + 1, i + 1, '+op' if up else '+ned'), cell, 'SENNEP Cell'))
            for (h0, h1, inner_up) in ((z_mod, z_mod + HT, True), (z_top - HT, z_top, False)):
                hb = G.box(x - px / 2 + 0.1, x + px / 2 - 0.1, y - py / 2 + 0.1, y + py / 2 - 0.1, h0, h1)
                # pocket for the cell from the inside, window through the lip on the outside
                pk0, pk1 = (h0 + lip, h1 + 1) if inner_up else (h0 - 1, h1 - lip)
                G.op(hb, G.tbm.createCylinderOrCone(Pt(x, y, pk0), (Q['Bat_CellDia'] / 2 + 0.2) * MM, Pt(x, y, pk1), (Q['Bat_CellDia'] / 2 + 0.2) * MM), D)
                G.op(hb, G.tbm.createCylinderOrCone(Pt(x, y, h0 - 1), 7.5 * MM, Pt(x, y, h1 + 1), 7.5 * MM), D)
                for (ex_, ey_) in ((-1, -1), (-1, 1), (1, -1), (1, 1)):
                    G.op(hb, G.tbm.createCylinderOrCone(Pt(x + ex_ * px / 2, y + ey_ * py / 2, h0 - 1), 1.1 * MM,
                                                        Pt(x + ex_ * px / 2, y + ey_ * py / 2, h1 + 1), 1.1 * MM), D)
                mod.append(('Holder %s %d-%d' % ('bund' if inner_up else 'top', j + 1, i + 1), hb, 'SENNEP Cell Holder'))

    # ---------- mount plate with a rim, and two clamp bridges ----------
    cl = Q['Bat_Clear']
    # every screw goes into a cross member web hole (x = 15, 45, 75 ...): plate at x=+-75, clamp feet at x=+-45
    gx, gy, bx = 75.0, (Y2 - Y1) / 2, 45.0
    plate = G.box(cx - gx - 7, cx + gx + 7, -Y2 - 12, -Y1 + 12, z_boss, z_mod)
    G.op(plate, G.box(-25.5, 25.5, -Y1 - 12.5, -Y1 + 13, z_boss - 1, z_mod + 1), D)     # room for the LiDAR mast foot
    rim = G.box(cx - L / 2 - cl - 3, cx + L / 2 + cl + 3, cy - W / 2 - cl - 3, cy + W / 2 + cl + 3, z_mod - 0.01, z_mod + 6)
    G.op(rim, G.box(cx - L / 2 - cl, cx + L / 2 + cl, cy - W / 2 - cl, cy + W / 2 + cl, z_mod - 1, z_mod + 7), D)
    G.op(plate, rim, U)
    screws = [(sx * gx, sy * gy) for sx in (-1, 1) for sy in (-1, 1)] + [(sx * bx, sy * gy) for sx in (-1, 1) for sy in (-1, 1)]
    for (dx, dy) in screws:
        G.op(plate, G.tbm.createCylinderOrCone(Pt(cx + dx, cy + dy, z_boss - 1), PM8['clear'] / 2 * MM, Pt(cx + dx, cy + dy, z_mod + 1), PM8['clear'] / 2 * MM), D)
    bridges = []
    leg0 = gy + PM8['w_od'] / 2 + 0.5           # legs outside the M8 washers
    leg1 = leg0 + 3
    foot0 = W / 2 + cl + 3 + 0.5               # feet start just outside the rim
    for sx in (-1, 1):
        x0, x1 = cx + sx * bx - 10, cx + sx * bx + 10
        br = G.box(x0, x1, cy - leg1, cy + leg1, z_top + 1.5, z_top + 6.5)             # bar, 1.5 mm above the strips
        for sy in (-1, 1):
            yl = sorted((cy + sy * leg0, cy + sy * leg1))
            G.op(br, G.box(x0, x1, yl[0], yl[1], z_mod, z_top + 6.5), U)               # leg
            yf = sorted((cy + sy * foot0, cy + sy * leg1))
            G.op(br, G.box(x0, x1, yf[0], yf[1], z_mod, z_mod + 3), U)                 # foot
            yp = sorted((cy + sy * (W / 2 - 2.5), cy + sy * (W / 2 - 0.05)))
            G.op(br, G.box(x0, x1, yp[0], yp[1], z_top, z_top + 1.6), U)               # pad on the holder edge
            # printed on its side (-X face down): the screw hole is horizontal in print -> teardrop, point +X
            G.op(br, G.teardrop((cx + sx * bx, cy + sy * gy, z_mod), (0, 0, 1), (1, 0, 0), PM8['clear'] / 2, -1, 4), D)
        bridges.append(br)

    # ---------- screws: through bridge foot + plate, or plate only, into the tray's inserts ----------
    hw = []
    for (dx, dy) in screws:
        through_foot = abs(abs(dx) - bx) < 0.1
        # foot + plate + web, or plate + web; the nut sits inside the cross member (reach it from below)
        hw += HWm.pbolt_stack((cx + dx, cy + dy, z_mod + (3 if through_foot else 0)), (0, 0, 1), (cx + dx, cy + dy, z_boss - Tw))

    tm, _ = target(des, '31 Battery Module', adsk.core.Matrix3D.create())
    write_bodies(des, tm, [(n, b, ap, True) for (n, b, ap) in mod])
    tb, _ = target(des, '32 Battery Mount', adsk.core.Matrix3D.create())
    write_bodies(des, tb, [('Battery Plate (print flat)', plate, 'SENNEP Chassis Part', True)] +
                 [('Battery Clamp %d (print on its side)' % (k + 1), b, 'SENNEP Chassis Part', True) for k, b in enumerate(bridges)] +
                 [('(skrue) %s' % n, b, 'SENNEP Chassis Part' if n.startswith('P-') else 'SENNEP Steel', True) for (n, b) in hw])

    # ---------- checks ----------
    tray = [b for b in des.allComponents.itemByName(CHASSIS).bRepBodies if b.name.startswith(('Cross', 'Rail', 'LiDAR'))]
    printed = [('plade', plate)] + [('klemme %d' % (k + 1), b) for k, b in enumerate(bridges)]
    modb = [b for (_, b, _) in mod]
    hits = []
    for (n, b) in printed:
        for mb in modb:
            if b.boundingBox.intersects(mb.boundingBox) and G.inter(b, mb) > 0.05:
                hits.append('%s/modul' % n)
                break
        for tb_ in tray:
            if b.boundingBox.intersects(tb_.boundingBox) and G.inter(b, tb_) > 0.05:
                hits.append('%s/%s' % (n, tb_.name))
        for (hn, hb) in hw:
            if not hn.startswith('Insert') and b.boundingBox.intersects(hb.boundingBox) and G.inter(b, hb) > 0.05:
                hits.append('%s/%s' % (n, hn))
    log.append('Kollisioner: ' + ('ingen' if not hits else ', '.join(sorted(set(hits)))))
    hh = []
    for tb_ in tray:
        for (hn, hb) in hw:
            if tb_.boundingBox.intersects(hb.boundingBox) and G.inter(tb_, hb) > 0.05:
                hh.append('%s/%s' % (tb_.name, hn))
    log.append('Skruer mod rammen: %s' % ('fri' if not hh else ', '.join(sorted(set(hh))[:6])))
    oh = overhangs(tb.bRepBodies.item(0), (0, 0, -1), z_boss, 2)
    log.append('Overhang plade (fladt): %s' % ('ingen' if not oh else '%d flader' % len(oh)))
    for k in (1, 2):
        bb = tb.bRepBodies.item(k).boundingBox
        oh = overhangs(tb.bRepBodies.item(k), (-1, 0, 0), bb.minPoint.x / MM, 0)
        log.append('Overhang klemme %d (paa siden): %s' % (k, 'ingen' if not oh else '%d flader' % len(oh)))
    log.append('Modul %.1f x %.1f x %.1f paa pladen mellem tvaerbjaelkerne y=-%.0f og -%.0f, toppen i z=%.1f' %
               (L, W, H, Y2, Y1, z_top))
    log.append('Skruer: 8x printet M8 ned i tvaerbjaelkerne (4 gennem klemmefod + plade, 4 kun plade), moetrik inde i bjaelken')
    finish_appearances(des)
    return '\n'.join(log)


def build_test_print():
    """'93 Test Print': a short piece of one cross-member joint (rail wall + cross member end, cut out of the real
    parts), one printed M8 bolt, nut and washers. Exported as STL to print/skelet_test/."""
    import os
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    Q = read_params(des)
    log = []
    G = Geo()
    HWm = Hardware(G)
    F = frame_numbers(des, Q, G, log)
    parts, _ = make_frame(G, Q, F)
    pm = dict(parts)
    y1, xr0 = Q['Frame_CrossY1'], F['xr0']
    rail = pm['Rail left front']
    G.op(rail, G.box(-F['xr1'] - 2, -xr0 + 2, -y1 - 30, -y1 + 30, F['Zb'] - 2, F['Zr'] + 5), G.I)
    cross = pm['Cross y-60 left']
    G.op(cross, G.box(-xr0 - 5, -xr0 + 40, -y1 - 25, -y1 + 25, F['Zb'] - 2, F['Zr'] + 5), G.I)
    # move the test pieces beside the robot
    m = adsk.core.Matrix3D.create()
    m.translation = V(-900 * MM, 0, 0)
    G.tbm.transform(rail, m)
    G.tbm.transform(cross, m)
    items = [('Test rail stykke (print toppen ned)', rail, 'SENNEP Tray', True),
             ('Test tvaerbjaelke ende (print toppen ned)', cross, 'SENNEP Tray', True),
             ('P-M8x%d bolt (print liggende paa fladen)' % PM8['L'], HWm.place(HWm.pbolt(), (-900, 60, 10), (1, 0, 0)), 'SENNEP Chassis Part', True),
             ('P-M8 moetrik (print fladt)', HWm.place(HWm.pnut(), (-870, 60, 0), (0, 0, 1)), 'SENNEP Chassis Part', True),
             ('P-M8 skive 1 (print fladt)', HWm.place(HWm.pwasher(), (-850, 60, 0), (0, 0, 1)), 'SENNEP Chassis Part', True),
             ('P-M8 skive 2 (print fladt)', HWm.place(HWm.pwasher(), (-830, 60, 0), (0, 0, 1)), 'SENNEP Chassis Part', True)]
    tc, _ = target(des, '93 Test Print', adsk.core.Matrix3D.create())
    write_bodies(des, tc, items)
    finish_appearances(des)
    folder = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'print', 'skelet_test')
    os.makedirs(folder, exist_ok=True)
    names = ['1x_Test_rail_stykke.stl', '1x_Test_tvaerbjaelke_ende.stl', '1x_P-M8x%d_bolt.stl' % PM8['L'], '1x_P-M8_moetrik.stl',
             '2x_P-M8_skive.stl']
    em = des.exportManager
    for i, n in enumerate(names):
        o = em.createSTLExportOptions(tc.bRepBodies.item(i), os.path.join(folder, n))
        o.meshRefinement = adsk.fusion.MeshRefinementSettings.MeshRefinementHigh
        em.execute(o)
    log.append('STL i %s: %s' % (folder, ', '.join(names)))
    return '\n'.join(log)


def build_body():
    """'50 Shell': the body. An octagonal (chamfered corners), facetted shell on top of the frame, sloped front
    with the 7" screen, triangle lattice in the flat top round the LiDAR, wheel arches, holes for 8 ToF,
    camera, lights, fans, speakers and the e-stop. v0 = one visual body (split into printable panels later)."""
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    if not app.activeDocument.name.startswith('SENNEP V2'):
        return 'STOP: aktivt dokument er "%s", ikke SENNEP V2' % app.activeDocument.name
    Q = read_params(des)
    log = []
    G = Geo()
    U, D, I_ = G.U, G.D, G.I
    F = frame_numbers(des, Q, G, log)
    W2, L2 = F['W2'], F['L2']
    t = Q['Shell_Wall']
    z0 = F['Zr']                               # sits on the frame (the bumpers stay visible below)
    zt = Q['Shell_Top']
    c = Q['Shell_Corner']
    e = Q['Shell_Chamfer']
    zf, sy_ = Q['Shell_FrontZ'], Q['Shell_SlopeY']
    r2 = 1 / math.sqrt(2)
    # front slope: from (y=-L2, z=zf) up to (y=-L2+sy_, z=zt)
    dz = zt - zf
    ln = math.hypot(sy_, dz)
    n_sl = (0, -dz / ln, sy_ / ln)             # outward normal of the slope
    u_sl = (0, sy_ / ln, dz / ln)              # up the slope

    def planes(off):
        """Half spaces of the shell's outside (off = 0) or inside (off = wall)."""
        P = []
        for sx in (-1, 1):
            P.append((sx * (W2 - off), 0, 0, sx, 0, 0))
            for sy in (-1, 1):
                P.append((sx * (W2 - c / 2) - sx * off * r2, sy * (L2 - c / 2) - sy * off * r2, 0, sx * r2, sy * r2, 0))
                # chamfer between the corner facet and the top
                n = (sx * 0.5, sy * 0.5, r2)
                P.append((sx * (W2 - c / 2) - n[0] * off, sy * (L2 - c / 2) - n[1] * off, zt - e - n[2] * off, n[0], n[1], n[2]))
            P.append((sx * (W2 - off * r2 * 0) - sx * off * r2, 0, zt - e - off * r2, sx * r2, 0, r2))
        for sy in (-1, 1):
            P.append((0, sy * (L2 - off), 0, 0, sy, 0))
        P.append((0, L2 - off * r2, zt - e - off * r2, 0, r2, r2))           # rear top chamfer
        P.append((0, 0, zt - off, 0, 0, 1))                                    # flat top
        P.append((0, -L2 - n_sl[1] * off, zf - n_sl[2] * off, n_sl[0], n_sl[1], n_sl[2]))   # front slope
        return P

    outer = G.box(-W2 - 1, W2 + 1, -L2 - 1, L2 + 1, z0, zt + 1)
    G.cut_by(outer, *planes(0))
    inner = G.box(-W2, W2, -L2, L2, z0 - 5, zt)
    G.cut_by(inner, *planes(t))
    shell = G.op(outer, inner, D)

    def at_slope(s, x=0.0):
        p = (x, -L2 + u_sl[1] * s, zf + u_sl[2] * s)
        return p

    cuts, show = [], []
    # wheel arches
    for sx in (-1, 1):
        for sy in (-1, 1):
            cuts.append(G.cyl(sx * (F['XW'] - 80), sx * (W2 + 5), Q['Shell_ArchR'], sy * F['YW'], F['ZW']))
    # LiDAR: the mast's flare and top plate come up through the top
    cuts.append(G.box(-30, 30, -28, 28, zt - t - 5, zt + 5))
    # 7" screen (Waveshare 7" HDMI (C): 164.9 x 100, view 155 x 87) in the front slope
    s_mid = ln * 0.45
    sc = at_slope(s_mid)
    scr = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(*sc), V(1, 0, 0), V(*u_sl), 156 * MM, 88 * MM, 20 * MM))
    cuts.append(scr)
    pc = (sc[0] - n_sl[0] * (t + 3), sc[1] - n_sl[1] * (t + 3), sc[2] - n_sl[2] * (t + 3))
    show.append(('(visning) 7" skaerm', G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(Pt(*pc), V(1, 0, 0), V(*u_sl), 164.9 * MM, 100 * MM, 6 * MM)), 'SENNEP Cell Holder'))
    # camera (Pi AI Camera) above the screen
    cc = at_slope(s_mid + 50 + 14)
    cuts.append(G.cyl_pts((cc[0] - n_sl[0] * 10, cc[1] - n_sl[1] * 10, cc[2] - n_sl[2] * 10), (cc[0] + n_sl[0] * 10, cc[1] + n_sl[1] * 10, cc[2] + n_sl[2] * 10), 7.0))
    # head and tail lights (3 W LED, 20 mm star)
    for sy in (-1, 1):
        for sx in (-1, 1):
            cuts.append(G.cyl_pts((sx * 200, sy * (L2 - 10), z0 + 35), (sx * 200, sy * (L2 + 10), z0 + 35), 11.0))
    # 8 ToF windows (VL53L0X), low: N, S, E, W and the 4 corner facets
    zt_ = z0 + 18
    tof = [((0, -L2, zt_), (0, -1, 0)), ((0, L2, zt_), (0, 1, 0)), ((-W2, 0, zt_), (-1, 0, 0)), ((W2, 0, zt_), (1, 0, 0))]
    for sx in (-1, 1):
        for sy in (-1, 1):
            tof.append(((sx * (W2 - c / 2), sy * (L2 - c / 2), zt_), (sx * r2, sy * r2, 0)))
    for (p, n) in tof:
        cuts.append(G.cyl_pts((p[0] - n[0] * 10, p[1] - n[1] * 10, p[2]), (p[0] + n[0] * 10, p[1] + n[1] * 10, p[2]), 5.0))

    def grille(cx_, cy_, cz_, n, d=38.0):
        """Round grille for a 40 mm fan / speaker: 5 slots inside a circle, through the wall along n."""
        cut = None
        for k in range(-2, 3):
            if abs(n[0]) > 0.5:
                b = G.box(cx_ - 15, cx_ + 15, cy_ - d / 2, cy_ + d / 2, cz_ + k * 7 - 1.75, cz_ + k * 7 + 1.75)
            else:
                b = G.box(cx_ - d / 2, cx_ + d / 2, cy_ - 15, cy_ + 15, cz_ + k * 7 - 1.75, cz_ + k * 7 + 1.75)
            cut = b if cut is None else G.op(cut, b, U)
        circ = G.cyl_pts((cx_ - n[0] * 15, cy_ - n[1] * 15, cz_), (cx_ + n[0] * 15, cy_ + n[1] * 15, cz_), d / 2)
        return G.op(cut, circ, I_)
    zm = (z0 + zt - e) / 2 + 10
    for sx in (-1, 1):
        cuts.append(grille(sx * W2, -60, zm, (sx, 0, 0)))          # fan intake, sides
        cuts.append(grille(sx * W2, 110, zm, (sx, 0, 0)))          # speakers, sides
        cuts.append(grille(sx * 150, L2, zm, (0, 1, 0)))           # fan exhaust, rear
    # e-stop (40 mm head, 22 mm panel hole) on the top, rear left
    cuts.append(G.tbm.createCylinderOrCone(Pt(-200, L2 - e - 60, zt - t - 2), 11 * MM, Pt(-200, L2 - e - 60, zt + 2), 11 * MM))
    show.append(('(visning) noedstop', G.tbm.createCylinderOrCone(Pt(-200, L2 - e - 60, zt), 20 * MM, Pt(-200, L2 - e - 60, zt + 18), 20 * MM), 'SENNEP Brass'))
    # triangle lattice in the flat top, either side of the LiDAR: panel minus 3 families of ribs (0/60/120 deg)
    for sx in (-1, 1):
        xa, xb = sorted((sx * 70, sx * (W2 - e - 30)))
        ya, yb = -L2 + sy_ + 40, L2 - e - 110
        panel = G.box(xa, xb, ya, yb, zt - t - 1, zt + 1)
        ribs = None
        pitch, rw = 22.0, 3.0
        span = math.hypot(xb - xa, yb - ya) + 10
        mx, my = (xa + xb) / 2, (ya + yb) / 2
        for ang in (0, 60, 120):
            a = math.radians(ang)
            dvec = V(math.cos(a), math.sin(a), 0)
            nvec = V(-math.sin(a), math.cos(a), 0)
            for k in range(-int(span / pitch), int(span / pitch) + 1):
                ctr = Pt(mx + nvec.x * k * pitch, my + nvec.y * k * pitch, zt - t / 2)
                b = G.tbm.createBox(adsk.core.OrientedBoundingBox3D.create(ctr, dvec, nvec, span * MM, rw * MM, (t + 4) * MM))
                ribs = b if ribs is None else G.op(ribs, b, U)
        G.op(ribs, G.box(xa + 4, xb - 4, ya + 4, yb - 4, zt - t - 2, zt + 2), I_)
        frame_ = G.op(G.box(xa, xb, ya, yb, zt - t - 2, zt + 2), G.box(xa + 4, xb - 4, ya + 4, yb - 4, zt - t - 3, zt + 3), D)
        holes = G.op(G.op(panel, ribs, D), frame_, D)
        cuts.append(holes)
    for cu in cuts:
        G.op(shell, cu, D)

    tgt, _ = target(des, '50 Shell', adsk.core.Matrix3D.create())
    write_bodies(des, tgt, [('Shell v0 (visuel, deles senere i print-paneler)', shell, 'SENNEP Shell', True)] +
                 [(n, b, ap, True) for (n, b, ap) in show])
    finish_appearances(des)

    # checks: against the frame, pods, wheels, motors, mast, LiDAR, battery
    others = []
    for cn in (CHASSIS, '31 Battery Module', '32 Battery Mount'):
        cmp_ = des.allComponents.itemByName(cn)
        if cmp_:
            others += [(b.name, b) for b in cmp_.bRepBodies]
    hits = []
    for (n, b) in others:
        if shell.boundingBox.intersects(b.boundingBox) and G.inter(shell, b) > 0.05:
            hits.append(n)
    log.append('Kollisioner med ramme/hjul/motorer/mast/batteri: %s' % ('ingen' if not hits else ', '.join(sorted(set(hits))[:10])))
    for (sx, sy, n) in [(-1, -1, 'FL'), (1, -1, 'FR'), (-1, 1, 'RL'), (1, 1, 'RR')]:
        env = G.cyl(sx * (F['XW'] - 40), sx * (F['XW'] + 40), F['P']['Rw'] + 4, sy * F['YW'], F['ZW'])
        if G.inter(shell, env) > 0.05:
            log.append('Hjul %s: skallen er inden for 4 mm af hjulet' % n)
    bb = shell.boundingBox
    scan = Q['BodyTop'] + Q['Lidar_Window']
    drop = math.hypot(W2, L2) * math.tan(math.radians(2))
    log.append('Skal: %.0f x %.0f x %.0f mm (z %.0f-%.0f), %.0f cm3 (~%.1f kg PETG massivt)' %
               ((bb.maxPoint.x - bb.minPoint.x) / MM, (bb.maxPoint.y - bb.minPoint.y) / MM, (bb.maxPoint.z - bb.minPoint.z) / MM,
                bb.minPoint.z / MM, bb.maxPoint.z / MM, shell.volume, shell.volume * 1.27 / 1000))
    log.append('LiDAR: scanningsplan z=%.1f, skallens top z=%.0f -> margen %.1f mm ved 2 grader i hjoernerne (skal vaere > 0)' %
               (scan, bb.maxPoint.z / MM, scan - drop - bb.maxPoint.z / MM))
    return '\n'.join(log)


def build():
    return build_pod()


def run(context):
    ui = adsk.core.Application.get().userInterface
    try:
        ui.messageBox(build_pod() + '\n\n' + build_chassis(), 'SENNEP chassisgenerator')
    except Exception:
        ui.messageBox(traceback.format_exc(), 'SENNEP chassisgenerator - fejl')
