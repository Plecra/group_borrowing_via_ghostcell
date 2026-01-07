mod ghost_cell;

use std::hash::Hash;

pub use ghost_cell::{GhostCell, GhostToken, InvariantLifetime};
pub use generativity::{make_guard, Guard};

/// GhostCell doesn't know about field project.
/// on a technicality this is no problem for these examples, since all signatures can just be extended to transitively
/// have lifetime parameters for all fields of the structure. However besides the obvious ergonomic issues with this,
/// it also doesnt support recursion correctly.
/// 
/// and the ergonomic issue is a real one: so here are a couple casts I've added. I believe they seem reasonable!
/// but please inspect them carefully. very curious to know any issues anyone finds.

/// Here's a special case of the rule below (entity_cast_group_mut) - it's less powerful, we can't use shared borrows inside a ghostcell.
/// but my current encoding of the existentials means that `GhostCell<Guard<'lt>, _>` can't be `GhostCell<r2, _>`
fn entity_cast_group_mut_<'a, r: EntityGroup, r2: EntityGroup>(_: &'a mut r, r: &'a mut Entity<r>) -> &'a mut Entity<r2> {
    unsafe {
        &mut *(r as *mut _ as *mut _)
    }
}
/// This is quite a cool rule if true, but definitely needs to be more
/// thoroughly inspected.
/// "when the r region contains r2, any object that can be accessed via 'r2 can be accessed via 'r"
/// this is used below for the union of groups.
/// 
/// 
// fn entity_cast_group_mut<'a, r: EntityGroup, r2: EntityGroup>(
//     x: &GhostCell<r, &mut r2>,
//     r: &'a mut Entity<r2>,
// ) -> &'a mut Entity<r> {
//     unsafe {
//         &mut *(r as *mut _ as *mut _)
//     }
// }
// /// You can see the definition of `OpenEntity` below. the intention is that these types are derived from every struct.
// /// here we make the claim that a GhostToken can be projected to the ghost tokens for every field of a type,
// /// where the function return rule is effectively implementing an existential: With system F, we'd give `Entity`
// /// an existential for the lifetimes on each field, and this function is opening it.
// /// 
// /// *definitely* suspicious of this particular signature. Does it make sense that the return lifetimes are unbound?
// /// `OpenEntity` makes then invariant, and we're confident in the uniqueness of `'r`, so this is splitting
// /// into exactly 5 ids? Generativity in rust is confusing, this might allow them to unify w something bad.
// fn token_as_entity1_mut<'r, 'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy, 'a>(t: &'a mut GhostToken<'r>) -> 
//     (&'a mut EntityAccess<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy>,
//     impl for<'b> Fn(&'b Entity<'r>) -> &'b OpenEntity<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> + 'r
//     ) {
//     (unsafe {
//         &mut *(t as *mut _ as *mut _)
//     },
//     // no captures ==> this is a zst
//     move |e| unsafe {
//         &*(e as *const _ as *const _)
//     }
//     )
// }




fn example() {
    let my_list: Vec<i64> = vec![1, 2, 3, 4];
    // "every variable introduces a group:"
    generativity::make_guard!(my_list_group);
    let mut my_list_group = my_list_group;
    let my_list = ghost_cell::GhostCell::new(my_list);
    let list_ref_a = &my_list;
    let list_ref_b = &my_list;
    list_ref_a.borrow_mut(&mut my_list_group).push(5);
    list_ref_b.borrow_mut(&mut my_list_group).push(6);
}
/// ```compile_fail
/// use demo::{GhostCell, make_guard};
/// fn example_2() {
///     let my_list = vec![1, 2, 3, 4];
///     make_guard!(my_list_group);
///     let mut my_list_group = my_list_group;
///     let my_list = GhostCell::new(my_list);
///     let list_ref = &my_list;
///     let el_ref = &my_list.borrow(&my_list_group)[0];
///     list_ref.borrow_mut(&mut my_list_group).push(5);
///     println!("{:?}", el_ref)
/// }
/// ```
fn example_2() {
    let my_list = vec![1, 2, 3, 4];
    generativity::make_guard!(my_list_group);
    let mut my_list_group = my_list_group;
    let my_list = ghost_cell::GhostCell::new(my_list);
    let list_ref = &my_list;
    let el_ref = &my_list.borrow(&my_list_group)[0];
    // list_ref.borrow_mut(&mut my_list_group).push(5);
    println!("{:?}", el_ref)
}
// Really these should all also be using GhostCell for the fields,
// but for the sake of the demo I'll just use plain old data.
#[derive(Debug)]
pub struct Ring {
    pub power: u32,
}
trait HandGroup {
    type DurabilityGroup;
    fn open_hand_mut<R>(&mut self, f: impl HandClosure<Self, Output = R>) -> R where Self: Sized;
}
struct HandToken<'id> {
    durability: InvariantLifetime<'id>,
}
impl<'dur> HandGroup for HandToken<'dur> {
    type DurabilityGroup = InvariantLifetime<'dur>;
    fn open_hand_mut<R>(&mut self, mut f: impl HandClosure<Self, Output = R>) -> R where Self: Sized {
        f.with_hand_token_mut(self, |h| h, |h| h)
    }
}
trait HandClosure<r> {
    type Output;
    fn with_hand_token_mut<'a, 'dur>(
        &mut self,
        token: &mut HandToken<'dur>,
        // Hands in this region have the `'dur` lifetime for their durability field
        f: impl (Fn(&Hand<r>) -> &Hand<HandToken<'dur>>) + 'a,
        // the token is still &mut self
        g: impl for<'l> Fn(&'l HandToken<'dur>) -> &'l r,
    ) -> Self::Output;
}
macro_rules! new_hand_closure {
    (<$($l:lifetime),*;$($T:ident : $($bounds:path),*),*> $r:ident [$($x : ident : $xt:ty),*] |$self:ident, $token:ident, $f:ident, $g : ident| $body:expr) => {{
        impl<$($l,)*$($T,)*> HandClosure<$r> for AnonHandClosure<$($l,)*$($T),*>
        where
            $r: HandGroup,
        {
            type Output = ();
            fn with_hand_token_mut<'f_xxxxx, 'dur>(
                &mut self,
                $token: &mut HandToken<'dur>,
                $f: impl (Fn(&Hand< $r >) -> &Hand<HandToken<'dur>>) + 'f_xxxxx,
                $g: impl for<'l> Fn(&'l HandToken<'dur>) -> &'l $r,
            ) -> () {
                let mut $self = self;
                $body
            }
        }
        struct AnonHandClosure<$($l,)*$($T : $($bounds +)*),*>($($xt),*);
        AnonHandClosure($($x),*)
    }};
}
pub enum Hand<r: HandGroup> {
    Shield {
        durability: GhostCell<r::DurabilityGroup, u32>,
    },
    Sword { sharpness: u32 },
}

struct EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H: HandGroup> {
    hp: InvariantLifetime<'hp>,
    rings: InvariantLifetime<'rings>,
    rings_content: InvariantLifetime<'rings_content>,
    hand: InvariantLifetime<'hand>,
    hand_content: H,
    energy: InvariantLifetime<'energy>,
}
// This is a closure with an HRTB, rust closure syntax can't help ;-;
trait EntityClosure<r> {
    type Output;
    fn with_entity_token_mut<'a, 'hp, 'rings, 'rings_content, 'hand, 'energy, H: HandGroup>(
        &mut self,
        token: &mut EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>,
        f: impl (Fn(&Entity<r>) -> &Entity<EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>>) + 'a
    ) -> Self::Output;
}
trait EntityGroup {
    type HpGroup;
    type RingsGroup;
    type RingsVecContentGroup;
    type HandGroup;
    type HandContentGroup: HandGroup;
    type EnergyGroup;
    fn open_entity_mut<R>(&mut self, f: impl EntityClosure<Self, Output = R>) -> R where Self: Sized;
}
macro_rules! new_entity_closure {
    (<$($l:lifetime),*;$($T:ident : $($bounds:path),*),*> $r:ident [$($x : ident : $xt:ty),*] |$self:ident, $token:ident, $f:ident| $body:expr) => {{
        impl<$($l,)*$($T,)*> EntityClosure<$r> for AnonEntityClosure<$($l,)*$($T),*>
        where
            r: EntityGroup,
        {
            type Output = ();
            fn with_entity_token_mut<'f_xxxxx, 'hp, 'rings, 'rings_content, 'hand, 'energy, H: HandGroup>(
                &mut self,
                $token: &mut EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>,
                $f: impl (Fn(&Entity<r>) -> &Entity<EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>>) + 'f_xxxxx
            ) -> () {
                // let AnonEntityClosure($($x),*) = self;
                let $self = self;
                $body
            }
        }
        struct AnonEntityClosure<$($l,)*$($T : $($bounds +)*),*>($($xt),*);
        AnonEntityClosure($($x),*)
    }};
}
impl<'hp, 'rings, 'rings_content, 'hand, 'energy, H> EntityGroup for EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>
where
    H: HandGroup,
{
    type HpGroup = InvariantLifetime<'hp>;
    type RingsGroup = InvariantLifetime<'rings>;
    type RingsVecContentGroup = InvariantLifetime<'rings_content>;
    type HandGroup = InvariantLifetime<'hand>;
    type HandContentGroup = H;
    type EnergyGroup = InvariantLifetime<'energy>;

    fn open_entity_mut<R>(&mut self, mut f: impl EntityClosure<Self, Output = R>) -> R where Self: Sized {
        f.with_entity_token_mut(self, |e| e)
    }
}
#[repr(C)]
pub struct Entity<r: EntityGroup> {
    pub hp: GhostCell<r::HpGroup, u32>,
    pub rings: GhostCell<r::RingsGroup, Vec<GhostCell<r::RingsVecContentGroup, Ring>>>,
    pub hand: GhostCell<r::HandGroup, Hand<r::HandContentGroup>>,
    pub energy: GhostCell<r::EnergyGroup, u32>,
}
// #[repr(C)]
// pub struct Entity<'content> {
//     pub hp: GhostCell<'content, u32>,
//     pub rings: GhostCell<'content, Vec<GhostCell<'content, Ring>>>,
//     pub hand: GhostCell<'content, Hand<'content>>,
//     pub energy: GhostCell<'content, i32>,
// }
// #[repr(C)]
// pub struct OpenEntity<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> {
//     pub hp: GhostCell<'hp, u32>,
//     pub rings: GhostCell<'rings, Vec<GhostCell<'rings_content, Ring>>>,
//     pub hand: GhostCell<'hand, Hand<'hand_content>>,
//     pub energy: GhostCell<'energy, u32>,
// }
// struct EntityAccess<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> {
//     pub hp: GhostToken<'hp>,
//     pub rings: GhostToken<'rings>,
//     pub rings_content: GhostToken<'rings_content>,
//     pub hand: GhostToken<'hand>,
//     pub hand_content: GhostToken<'hand_content>,
//     pub energy: GhostToken<'energy>,
// }

impl<r: EntityGroup> Entity<r> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn calculate_damage(&self, other: &Entity<r>, access: &r) -> u32 {
        todo!()
    }
    pub fn calculate_attack_cost(&self, other: &Entity<r>, access: &r) -> u32{
        todo!()
    }
    pub fn calculate_defend_cost(&self, other: &Entity<r>, access: &r) -> u32 {
        todo!()
    }
    pub fn use_energy(&self, cost: u32, access: &mut r) {
        todo!()
    }
    pub fn damage(&self, cost: u32, access: &mut r) {
        todo!()
    }
}

fn attack<r: EntityGroup>(a: &Entity<r>, d: &Entity<r>, token: &mut r) {
    let damage = a.calculate_damage(d, token);
    let a_energy_cost = a.calculate_attack_cost(d, token);
    let d_energy_cost = d.calculate_defend_cost(a, token);
    a.use_energy(a_energy_cost, token);
    d.use_energy(d_energy_cost, token);
    d.damage(damage, token);
}
/// ```compile_fail
/// use demo::{GhostToken, Entity};
/// fn attack2<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
///     let hp = &a.hp;
///     let ring_ref = &a.rings.borrow(&token)[0];
///     
///     let damage = a.calculate_damage(d, token);
///     let a_energy_cost = a.calculate_attack_cost(d, token);
///     let d_energy_cost = d.calculate_defend_cost(a, token);
///     a.use_energy(a_energy_cost, token);
///     d.use_energy(d_energy_cost, token);
///     d.damage(damage , token);
///     println!("{:?}", hp.borrow(&token));
///     println!("{:?}", ring_ref);
/// }
/// ```
/// 
/// demonstrating accessing field content and child groups
pub fn attack2<r: EntityGroup>(a: &Entity<r>, d: &Entity<r>, token: &mut r) {
    token.open_entity_mut(new_entity_closure!(<'a;r: EntityGroup> r[a : &'a Entity<r>, d : &'a Entity<r>] |s, token, f| {
        let a = f(s.0);
        let d = f(s.1);
        let hp = &a.hp;
        let ring_ref = &a.rings.borrow(&token.rings)[0];
        
        let damage = a.calculate_damage(d, token);
        let a_energy_cost = a.calculate_attack_cost(d, token);
        let d_energy_cost = d.calculate_defend_cost(a, token);
        a.use_energy(a_energy_cost, token);
        d.use_energy(d_energy_cost, token);
        d.damage(damage , token);
        println!("{:?}", hp.borrow(&token.hp));
        // println!("{:?}", ring_ref);
    }));
}

// fn attack3<r: EntityGroup>(a: &Entity<r>, d: &Entity<r>, token: &mut GhostToken<r>) {
//     token.open_entity_mut(new_entity_closure!([] |s, t, f| {
//         let a = f(a);
//         let b = f(b);
//         let hp_ref = &d.hp;
//         let rings_list_ref = &d.rings;
//         let rand_n = a as *const _ as usize;
//         let ring_ref = &d.rings.borrow(&token)[rand_n];

//         let durability = match &*d.hand.borrow(&token) {
//             Hand::Shield { durability } => {
//                 durability
//             }
//             Hand::Sword { sharpness } => {
//                 panic!("irrelevant to the demo :)");
//             }
//         };
//         println!("{:?}", hp_ref.borrow(&token));
//         println!("{:?}", rings_list_ref.borrow(&token).len());
//         println!("{:?}", ring_ref.borrow(&token).power);
//         println!("{:?}", durability.borrow(&token));
//     }
// }
/// ```compile_fail
/// use demo::{GhostToken, Entity, Hand};
/// fn attack4<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
///     let hp_ref = &d.hp;
///     let rings_list_ref = &d.rings;
///     let rand_n = a as *const _ as usize;
///     let ring_ref = &d.rings.borrow(&token)[rand_n];
/// 
///     let durability = match &*d.hand.borrow(&token) {
///         Hand::Shield { durability } => {
///             durability
///         }
///         Hand::Sword { sharpness } => {
///             panic!("irrelevant to the demo :)");
///         }
///     };
///     d.damage(10, token);
///     println!("{:?}", hp_ref.borrow(&token));
///     println!("{:?}", rings_list_ref.borrow(&token).len());
///     println!("{:?}", ring_ref.borrow(&token).power);
///     println!("{:?}", durability);
/// }
/// ```
fn attack4<r: EntityGroup>(a: &Entity<r>, d: &Entity<r>, token: &mut r) {
    token.open_entity_mut(new_entity_closure!(<'a;r: EntityGroup> r[a : &'a Entity<r>, d : &'a Entity<r>] |s, token, f| {
        let a = f(s.0);
        let d = f(s.1);
        let hp_ref = &d.hp;
        let rings_list_ref = &d.rings;
        let rand_n = a as *const _ as usize;
        let ring_ref = &d.rings.borrow(&token.rings)[rand_n];
        let durability = match &*d.hand.borrow(&token.hand) {
            Hand::Shield { durability } => {
                durability
            }
            Hand::Sword { sharpness } => {
                panic!("irrelevant to the demo :)");
            }
        };
        d.damage(10, token);
        println!("{:?}", hp_ref.borrow(&token.hp));
        println!("{:?}", rings_list_ref.borrow(&token.rings).len());
        // println!("{:?}", ring_ref.borrow(&token.rings_content).power);
        // println!("{:?}", durability);
    }));

}
// fn union_groups<'r>(a: &mut GhostToken<'a>, b: &mut GhostToken<'b>, r: GhostToken<'r>) ->  {
//     GhostToken::new(id)
// }
// struct And<A, B>(A, B);
impl<A: EntityGroup, B> EntityGroup for (A, B) {
    type HpGroup = A::HpGroup;
    type RingsGroup = A::RingsGroup;
    type RingsVecContentGroup = A::RingsVecContentGroup;
    type HandGroup = A::HandGroup;
    type HandContentGroup = A::HandContentGroup;
    type EnergyGroup = A::EnergyGroup;

    fn open_entity_mut<R>(&mut self, f: impl EntityClosure<Self, Output = R>) -> R where Self: Sized {
        struct AnonClosure<A, B, R, F: EntityClosure<(A, B), Output = R>>(F, core::marker::PhantomData<fn(A, B) -> R>);
        impl<A: EntityGroup, B, R, F: EntityClosure<(A, B), Output = R>> EntityClosure<A> for AnonClosure<A, B, R, F> {
            type Output = R;
            fn with_entity_token_mut<'a, 'hp, 'rings, 'rings_content, 'hand, 'energy, H: HandGroup>(
                &mut self,
                token: &mut EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>,
                f: impl (Fn(&Entity<A>) -> &Entity<EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>>) + 'a
            ) -> R {
                let AnonClosure(inner_f, _marker) = self;
                inner_f.with_entity_token_mut(token, |e_a_b| {
                    // let e_a: &Entity<A> = unsafe { &*(e_a_b as *const _ as *const _) };
                    // f(e_a)
                    todo!()
                })
            }
        }
        let closure = AnonClosure::<A, B, R, _>(f, core::marker::PhantomData);
        self.0.open_entity_mut(closure)
    }
}

fn invoke_demo() {
    let entity_a = Entity::new();
    generativity::make_guard!(hp_token);
    generativity::make_guard!(rings_token);
    generativity::make_guard!(rings_content_token);
    generativity::make_guard!(hand_token);
    generativity::make_guard!(durability_token);
    generativity::make_guard!(energy_token);    
    let mut entity_a_content_group = EntityToken {
        hp: hp_token,
        rings: rings_token,
        rings_content: rings_content_token,
        hand: hand_token,
        hand_content: HandToken {
            durability: durability_token,
        },
        energy: energy_token,
    };
    // let mut entity_a_content_group = entity_a_content_group;
    generativity::make_guard!(entity_a_group);
    let mut entity_a_group = entity_a_group;
    let entity_a = ghost_cell::GhostCell::new(entity_a);

    let entity_b = Entity::new();
    generativity::make_guard!(hp_token);
    generativity::make_guard!(rings_token);
    generativity::make_guard!(rings_content_token);
    generativity::make_guard!(hand_token);
    generativity::make_guard!(durability_token);
    generativity::make_guard!(energy_token);    
    let mut entity_b_content_group = EntityToken {
        hp: hp_token,
        rings: rings_token,
        rings_content: rings_content_token,
        hand: hand_token,
        hand_content: HandToken {
            durability: durability_token,
        },
        energy: energy_token,
    };
    generativity::make_guard!(entity_b_group);
    let mut entity_b_group = entity_b_group;
    let entity_b = ghost_cell::GhostCell::new(entity_b);
    _ = entity_a.borrow_mut(&mut entity_a_group).damage(0, &mut entity_a_content_group);
    _ = entity_b.borrow_mut(&mut entity_b_group).damage(0, &mut entity_b_content_group);
    attack(
        entity_a.borrow(&entity_a_group),
        entity_b.borrow(&entity_b_group),
        // could eg try to pass just one of their groups,
        // which rust will unify with the `'r` on both entities.
        // so we actually get an error from trying to extend 
        // entity_b_content_group to live as long as entity_a_group.
        todo!("wont work: &mut entity_b_content_group")
        
    );
    generativity::make_guard!(hp_token);
    generativity::make_guard!(rings_token);
    generativity::make_guard!(rings_content_token);
    generativity::make_guard!(hand_token);
    generativity::make_guard!(durability_token);
    generativity::make_guard!(energy_token);
    generativity::make_guard!(ghost_cell_holding_content_groups);
    let a_b_content_union = ((EntityToken {
        hp: hp_token,
        rings: rings_token,
        rings_content: rings_content_token,
        hand: hand_token,
        hand_content: HandToken {
            durability: durability_token,
        },
        energy: energy_token,
    }, ghost_cell_holding_content_groups));
    let entity_a = entity_a.borrow_mut(&mut entity_a_group);
    let token = &mut (a_b_content_union.0);
    attack(
        entity_cast_group_mut_(&mut entity_a_content_group, entity_a),
        entity_cast_group_mut_(&mut entity_a_content_group, entity_b.borrow_mut(&mut entity_b_group)),
        &mut a_b_content_union
    );
}

fn complex_example_main() {
    let entities = vec![
        GhostCell::new(Entity::new()),
        GhostCell::new(Entity::new()),
    ];
    generativity::make_guard!(hp_token);
    generativity::make_guard!(rings_token);
    generativity::make_guard!(rings_content_token);
    generativity::make_guard!(hand_token);
    generativity::make_guard!(durability_token);
    generativity::make_guard!(energy_token);    
    let mut entity_content_group = EntityToken {
        hp: hp_token,
        rings: rings_token,
        rings_content: rings_content_token,
        hand: hand_token,
        hand_content: HandToken {
            durability: durability_token,
        },
        energy: energy_token,
    };
    generativity::make_guard!(entities_content_group);
    let mut entities_content_group = entities_content_group;
    generativity::make_guard!(entities_group);
    let mut entities_group = entities_group;
    let entities = GhostCell::new(entities);
    let x = entities.borrow(&entities_group)[0].borrow(&entities_content_group);
    attack(
        x,
        entities.borrow(&entities_group)[1].borrow(&entities_content_group),
        &mut entity_content_group
    );
}
//fn attack[mut r: group Entity](
//    ref[r] a: Entity,
//    ref[r] d: Entity):
//  ref armor_ref = a.armor # Ref to a's armor
//
//  # Modifies a.rings' contents
//  power_up_ring(a, a.rings[0])
//
//  # Valid, compiler knows we only modified a.rings' contents
//  armor_ref.hardness += 2
fn complex_attack2<r: EntityGroup>(a: &Entity<r>, d: &Entity<r>, token: &mut r) {
    token.open_entity_mut(new_entity_closure!(<'a;r : EntityGroup> r [a : &'a Entity<r>, d : &'a Entity<r>] |s, token, f| {
        let a = f(s.0);
        let d = f(s.1);
        let EntityToken { hp, rings, hand, hand_content, energy, rings_content } = token;
        hand_content.open_hand_mut(new_hand_closure!(<'a, 'b, 'hp, 'rings, 'rings_content, 'hand, 'energy;H : HandGroup> H [
            a : &'a Entity<EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>>
            , hp : &'a InvariantLifetime<'hp>,
            rings : &'a InvariantLifetime<'rings>,
            hand: &'a InvariantLifetime<'hand>,
            // hand_content: &'a H,
            energy : &'a InvariantLifetime<'energy>,
            rings_content : &'b mut InvariantLifetime<'rings_content>
            ] |s, hand_token, f, g| {
                // let hand = f(s.0);
                let a = s.0;
                let hp = s.1;
                let rings = s.2;
                let hand = s.3;
                let hand_content = f(&*a.hand.borrow(&hand));
                let energy = s.4;
                let rings_content = &mut *s.5;
                let armor_ref = match &hand_content {
                    Hand::Shield { durability } => {
                        durability
                    }
                    Hand::Sword { sharpness } => {
                        panic!("irrelevant to the demo :)");
                    }
                };
                let rc = a.rings.borrow(&rings)[0].borrow_mut(rings_content);
                complex_power_up_ring(
                    a, 
                    rc,
                    &hp,
                    &rings,
                    // &token.rings_content,
                    &hand,
                    g(hand_token),
                    &energy,
                );
                // token.hand_content.open_hand_mut(new_hand_closure!(<'a;r> r [armor_ref]))
                *armor_ref.borrow_mut(&mut hand_token.durability) += 2;
        }));
    }));
}
// # Wielder Entity's energy will power up the ring.
// # Changes the ring, but does not change the wielder Entity.
// fn complex_power_up_ring[e: group Entity, mut rr: group Ring = e.rings*](
//     ref[e] entity: Entity,
//     ref[rr] a_ring: Ring
// ):
fn complex_power_up_ring<'hp, 'rings, 'rings_content, 'hand, 'energy, H: HandGroup>(
    entity: &Entity<EntityToken<'hp, 'rings, 'rings_content, 'hand, 'energy, H>>,
    a_ring: &mut Ring,

    // So rust can't reason about borrows already existing in the sigature
    // - e.rings is lovely - but we can mimic it by just exhaustively listing
    // the disjunction
    token1: &InvariantLifetime<'hp>,
    token2: &InvariantLifetime<'rings>,
    token3: &InvariantLifetime<'hand>,
    token4: &H,
    token5: &InvariantLifetime<'energy>,
) {
    a_ring.power += entity.energy.borrow(token5) / 4
}