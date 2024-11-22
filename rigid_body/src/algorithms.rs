use crate::joint::{Joint, JointType};
use crate::sva::{Force, Motion, Xform};
use bevy::prelude::*;

pub fn loop_1_update(joint: &mut Joint, parent: &Joint) {
    // reset joint
    joint.tau = 0.;
    joint.f_ext = Force::zero();
    joint.qdd = 0.;
    joint.a = Motion::zero();

    // joint transform
    joint.xj = match joint.joint_type {
        JointType::Base => Xform::identity(),
        JointType::Rx => Xform::rotx(joint.q),
        JointType::Ry => Xform::roty(joint.q),
        JointType::Rz => Xform::rotz(joint.q),
        JointType::Px => Xform::posx(joint.q),
        JointType::Py => Xform::posy(joint.q),
        JointType::Pz => Xform::posz(joint.q),
    };

    joint.vj = joint.qd * joint.s;
    joint.xl = joint.xj * joint.xt;

    joint.x = joint.xl * parent.x;
    joint.v = (joint.xl * parent.v) + joint.vj;

    joint.c = joint.v.cross_v(joint.vj);
    joint.iaa = joint.i.into();
    joint.paa = joint.v.cross_f(joint.i * joint.v);
}

pub fn apply_external_update(joint: &mut Joint, _parent: &Joint) {
    joint.paa -= joint.x * joint.f_ext;
}

pub fn loop_2_update(joint: &mut Joint, parent_option: Option<&mut Joint>) {
    joint.uu = joint.iaa * joint.s;
    joint.dd = joint.s.w.dot(&joint.uu.m) + joint.s.v.dot(&joint.uu.f);
    joint.u = joint.tau - (joint.s.w.dot(&joint.paa.m) + joint.s.v.dot(&joint.paa.f));

    match parent_option {
        None => {}
        Some(parent) => {
            let dd_inv = 1. / joint.dd;
            let ia = joint.iaa - (dd_inv * joint.uu.self_outer_product());
            let pa = joint.paa + (ia * joint.c) + ((dd_inv * joint.u) * joint.uu);
            let xli = joint.xl.inverse();
            parent.iaa += xli * ia;
            parent.paa += xli * pa;
        }
    }
}

pub fn loop_3_update(joint: &mut Joint, parent: &Joint) {
    let ap = joint.xl * parent.a + joint.c;

    let dd_inv = 1. / joint.dd;
    let te = joint.u - (joint.uu.m.dot(&ap.w) + joint.uu.f.dot(&ap.v));
    joint.qdd = dd_inv * te;
    joint.a = ap + (joint.qdd * joint.s);
}

pub fn integrate_joint_state(fixed_time: Res<FixedTime>, mut joint_query: Query<&mut Joint>) {
    let dt = fixed_time.period.as_secs_f64();
    for mut joint in joint_query.iter_mut() {
        joint.q += joint.qd * dt;
        joint.qd += joint.qdd * dt;
    }
}
