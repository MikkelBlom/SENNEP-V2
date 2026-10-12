"""Hardware for one wheel corner (21 Pod Lab), so the assembly can be seen in Fusion.

Run via MCP after exec'ing SENNEP_ChassisGenerator.py in the same namespace:
    exec(open(<this file>).read()); print(build_pod_hardware())

'22 Pod Lab Hardware'  : every screw, nut, washer and insert in its final position.
'23 Pod Lab Skruetraekkere bagfra' : (hidden) two Oe8 screwdrivers reaching the bearing-keeper screws
                         through the tool channels from the motor end, with the motor in place.
"""

SET_SCREW_L = 12     # mm, shortest M3 in the kit (a short grub screw is better if one turns up)


def build_pod_hardware():
    app = adsk.core.Application.get()
    des = adsk.fusion.Design.cast(app.activeProduct)
    Q = read_params(des)
    G = Geo()
    I = wheel_interface(des, Q)
    pod, collar, keeper, P = make_pod(G, Q, I, [])
    HWm = Hardware(G)
    ang = math.radians(Q['Chassis_BoltAngle'])
    log = []

    def bolt_pt(x, sy):
        return (x, sy * P['hl'] * math.cos(ang), sy * P['hl'] * math.sin(ang))

    ex = []
    # bearing keeper: M3x19 + washer from BEHIND the flange (head in the pocket floor), into brass inserts
    # pressed into the keeper's wheel-side face
    for sy in (-1, 1):
        for (n, b) in HWm.bolt_stack(3, 19, bolt_pt(P['x_head_seat'], sy), (1, 0, 0)):
            ex.append(('Lejeholder: ' + n, b))
        ex.append(('Lejeholder: Insert M3 (i holderen)', HWm.place(HWm.insert(), bolt_pt(P['x_keep0'], sy), (-1, 0, 0))))
    # motor collar: 4x M4x12 + washers into the motor's own threads
    for k in range(4):
        a = math.radians(45 + 90 * k)
        for (n, b) in HWm.bolt_stack(4, 12, (P['x_coll0'], Q['Mot_HolePCD'] / 2 * math.cos(a), Q['Mot_HolePCD'] / 2 * math.sin(a)), (-1, 0, 0)):
            ex.append(('Motorkrave: ' + n, b))
    # coupling: captive M3 nut flush with the bore's D-flat, screw from the top onto the shaft flat
    up = des.userParameters
    shaft_r = up.itemByName('Motor_ShaftDia').value / MM / 2
    flat = up.itemByName('Motor_ShaftFlat').value / MM
    clr = up.itemByName('Motor_ShaftClear').value / MM
    z_flat_bore = shaft_r + clr - flat
    ex.append(('Kobling: Nut M3', HWm.place(HWm.nut(3), (I['x_set'], 0, z_flat_bore), (0, 0, 1))))
    ex.append(('Kobling: M3x%d (pinolskrue, spaendt mod D-fladen)' % SET_SCREW_L,
               HWm.place(HWm.screw(3, SET_SCREW_L), (I['x_set'], 0, shaft_r - flat + SET_SCREW_L), (0, 0, 1))))

    # ---------- checks ----------
    printed = [('pod', pod), ('krave', collar), ('holder', keeper)]
    hits = []
    for (hn, hb) in ex:
        if 'Insert' in hn:
            continue
        for (pn, pb) in printed:
            v = G.inter(hb, pb)
            if v > 0.05:
                hits.append('%s/%s %.1f' % (hn, pn, v))
    log.append('Hardware mod printede dele (mm3): %s' % ('fri' if not hits else '; '.join(hits)))
    # the set screw turns with the wheel: sweep it once round the axle against the pod
    ss = [b for (n, b) in ex if n.startswith('Kobling: M3x')][0]
    worst = 0.0
    for k in range(36):
        c = G.rot(G.tbm.copy(ss), 10 * k)
        worst = max(worst, G.inter(c, pod))
    log.append('Pinolskruen M3x%d drejet 360 grader mod poden: %s' % (SET_SCREW_L, 'fri' if worst < 0.05 else 'RAMMER %.1f mm3' % worst))

    wocc = [o for o in des.rootComponent.occurrences if o.component.name == WHEEL][0]
    hc, hocc = target(des, '22 Pod Lab Hardware', wocc.transform2)
    write_bodies(des, hc, [(n, b, 'SENNEP Brass' if 'Insert' in n else 'SENNEP Steel', True) for (n, b) in ex])

    # ---------- screwdrivers from the back, with the motor in place ----------
    head_top = P['x_head_seat'] + HW[3]['w_t'] + HW[3]['k'] * 0.6        # inside the hex socket
    drivers = [('Skruetraekker 2,5 mm unbrako (Oe8 skaft)', G.cyl(head_top, I['x_rear'] + 80.0, 4.0, *bolt_pt(0, sy)[1:]))
               for sy in (-1, 1)]
    blockers = [(t, b) for (t, b) in display_bodies(des, I, wheel=False)] + [('housing', pod), ('krave', collar)]
    blockers += [(n, b) for (n, b) in ex if 'Motorkrave' in n]
    dh = []
    for (dn, db) in drivers:
        for (bn, bb) in blockers:
            if G.inter(db, bb) > 0.05:
                dh.append(bn)
    log.append('Skruetraekker Oe8 bagfra til lejeholder-skruerne, med motoren i: %s' % ('fri' if not dh else 'RAMMER ' + ', '.join(sorted(set(dh)))))
    sc, socc = target(des, '23 Pod Lab Skruetraekkere bagfra', wocc.transform2)
    write_bodies(des, sc, [(n, b, 'SENNEP Steel', True) for (n, b) in drivers])
    finish_appearances(des)
    socc.isLightBulbOn = False
    log.append('Lejeholder-skruer: skive paa x = %.1f, %.1f mm bund under hovedet' % (P['x_head_seat'], P['x_head_seat'] - I['x_back']))
    return '\n'.join(log)
