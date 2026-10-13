"""Print files for the minimum rolling chassis: universal rail, universal cross-member half, V-key bar and
the flanged printed M8 bolt + nut. Every part is turned to its print orientation (face on the bed at z = 0).

Run via MCP after exec'ing SENNEP_ChassisGenerator.py in the same namespace:
    exec(open(<this file>).read()); print(build_chassis_print())
"""
import os

PRINT_DIR = os.path.join(r'C:\Users\mikke\Desktop\Projects\SENNEP V2', 'print', 'chassis_v1')


def build_chassis_print():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    Q = read_params(des)
    log = []
    G = Geo()
    HWm = Hardware(G)
    F = frame_numbers(des, Q, G, log)
    parts, _ = make_frame(G, Q, F)
    pm = dict(parts)

    def to_bed(body, rot=None):
        """Rotate (axis, deg) about the origin, then put the lowest point on z = 0 and centre it in x/y."""
        if rot:
            G.rot(body, rot[1], rot[0])
        bb = body.boundingBox
        m = adsk.core.Matrix3D.create()
        m.translation = V(-(bb.minPoint.x + bb.maxPoint.x) / 2, -(bb.minPoint.y + bb.maxPoint.y) / 2, -bb.minPoint.z)
        G.tbm.transform(body, m)
        return body

    s = Q['Frame_KeyDepth'] * math.sqrt(2)
    kl = KEY_L
    items = [
        ('4x_Rail (universal)', to_bed(G.tbm.copy(pm['Rail left front']), ((1, 0, 0), 180))),
        ('4x_Cross_half (universal)', to_bed(G.tbm.copy(pm['Cross y-60 left']), ((1, 0, 0), 180))),
        ('4x_Key_bar', to_bed(G.box(-kl / 2, kl / 2, -s / 2, s / 2, 0, s))),
        ('20x_P-M8x%d_bolt' % PM8['L'], to_bed(HWm.pbolt(), ((1, 0, 0), -90))),     # lying on one of its flats
        ('20x_P-M8_nut', to_bed(HWm.pnut())),                                          # flange on the bed
    ]
    # beside each other in Fusion, for a look (hidden component)
    shown = []
    for i, (n, b) in enumerate(items):
        c = G.tbm.copy(b)
        m = adsk.core.Matrix3D.create()
        m.translation = V(-1200 * MM, (i * 260 - 500) * MM, 0)
        G.tbm.transform(c, m)
        shown.append((n, c, 'SENNEP Tray' if 'Rail' in n or 'Cross' in n else 'SENNEP Chassis Part', True))
    tc, tocc = target(des, '94 Chassis Print (minimum)', adsk.core.Matrix3D.create())
    write_bodies(des, tc, shown)
    finish_appearances(des)
    tocc.isLightBulbOn = False

    # export the oriented bodies through a temporary component at the origin
    os.makedirs(PRINT_DIR, exist_ok=True)
    xc, xocc = target(des, '94 Chassis Print (export tmp)', adsk.core.Matrix3D.create())
    write_bodies(des, xc, [(n, b, None, True) for (n, b) in items])
    em = des.exportManager
    for i, (n, b) in enumerate(items):
        fn = n.replace(' (universal)', '') + '.stl'
        o = em.createSTLExportOptions(xc.bRepBodies.item(i), os.path.join(PRINT_DIR, fn))
        o.meshRefinement = adsk.fusion.MeshRefinementSettings.MeshRefinementHigh
        o.isBinaryFormat = True
        o.sendToPrintUtility = False
        em.execute(o)
        bb = b.boundingBox
        log.append('%s: %.1f x %.1f x %.1f mm, %.0f cm3' % (fn, (bb.maxPoint.x - bb.minPoint.x) / MM, (bb.maxPoint.y - bb.minPoint.y) / MM,
                                                            (bb.maxPoint.z - bb.minPoint.z) / MM, b.volume))
    xocc.deleteMe()
    # print-direction check on the beams (bed face at z = 0, up = +z)
    for (n, b) in items[:2]:
        tmp_c, tmp_o = target(des, '94 tmp check', adsk.core.Matrix3D.create())
        write_bodies(des, tmp_c, [(n, b, None, True)])
        bad = overhangs(tmp_c.bRepBodies.item(0), (0, 0, -1), 0.0, 2)
        log.append('Overhang %s: %s' % (n, 'ingen' if not bad else '%d flader (fx %.0f mm2)' % (len(bad), bad[0][0])))
        tmp_o.deleteMe()
    del PENDING[:]          # the temporary components are gone; don't re-apply appearances to them later
    return '\n'.join(log)
