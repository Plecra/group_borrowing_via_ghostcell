mod ghost_cell;

pub use ghost_cell::{GhostCell, GhostToken};
pub use generativity::{make_guard, Guard};

/// GhostCell doesn't know about field project.
/// on a technicality this is no problem for these examples, since all signatures can just be extended to transitively
/// have lifetime parameters for all fields of the structure. However besides the obvious ergonomic issues with this,
/// it also doesnt support recursion correctly.
/// 
/// and the ergonomic issue is a real one: so here are a couple casts I've added. I believe they seem reasonable!
/// but please inspect them carefully. very curious to know any issues anyone finds.

/// This is quite a cool rule if true, but definitely needs to be more
/// thoroughly inspected.
/// "when the r region contains r2, any object that can be accessed via 'r2 can be accessed via 'r"
/// this is used below for the union of groups.
fn entity_cast_group_mut<'r, 'r2, 'a>(
    x: &GhostCell<'r, &mut GhostToken<'r2>>,
    r: &'a mut Entity<'r2>,
) -> &'a mut Entity<'r> {
    unsafe {
        &mut *(r as *mut _ as *mut _)
    }
}
/// You can see the definition of `OpenEntity` below. the intention is that these types are derived from every struct.
/// here we make the claim that a GhostToken can be projected to the ghost tokens for every field of a type,
/// where the function return rule is effectively implementing an existential: With system F, we'd give `Entity`
/// an existential for the lifetimes on each field, and this function is opening it.
/// 
/// *definitely* suspicious of this particular signature. Does it make sense that the return lifetimes are unbound?
/// `OpenEntity` makes then invariant, and we're confident in the uniqueness of `'r`, so this is splitting
/// into exactly 5 ids? Generativity in rust is confusing, this might allow them to unify w something bad.
fn token_as_entity1_mut<'r, 'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy, 'a>(t: &'a mut GhostToken<'r>) -> 
    (&'a mut EntityAccess<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy>,
    impl for<'b> Fn(&'b Entity<'r>) -> &'b OpenEntity<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> + 'r
    ) {
    (unsafe {
        &mut *(t as *mut _ as *mut _)
    },
    // no captures ==> this is a zst
    move |e| unsafe {
        &*(e as *const _ as *const _)
    }
    )
}




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
pub enum Hand<'content> {
    Shield {
        durability: GhostCell<'content, u32>,
    },
    Sword { sharpness: u32 },
}
#[repr(C)]
pub struct Entity<'content> {
    pub hp: GhostCell<'content, u32>,
    pub rings: GhostCell<'content, Vec<GhostCell<'content, Ring>>>,
    pub hand: GhostCell<'content, Hand<'content>>,
    pub energy: GhostCell<'content, i32>,
}
#[repr(C)]
pub struct OpenEntity<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> {
    pub hp: GhostCell<'hp, u32>,
    pub rings: GhostCell<'rings, Vec<GhostCell<'rings_content, Ring>>>,
    pub hand: GhostCell<'hand, Hand<'hand_content>>,
    pub energy: GhostCell<'energy, u32>,
}
struct EntityAccess<'hp, 'rings, 'rings_content, 'hand, 'hand_content, 'energy> {
    pub hp: GhostToken<'hp>,
    pub rings: GhostToken<'rings>,
    pub rings_content: GhostToken<'rings_content>,
    pub hand: GhostToken<'hand>,
    pub hand_content: GhostToken<'hand_content>,
    pub energy: GhostToken<'energy>,
}

impl<'r> Entity<'r> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn calculate_damage(&self, other: &Entity<'r>, access: &GhostToken<'r>) -> u32 {
        todo!()
    }
    pub fn calculate_attack_cost(&self, other: &Entity<'r>, access: &GhostToken<'r>) -> u32{
        todo!()
    }
    pub fn calculate_defend_cost(&self, other: &Entity<'r>, access: &GhostToken<'r>) -> u32 {
        todo!()
    }
    pub fn use_energy(&self, cost: u32, access: &mut GhostToken<'r>) {
        todo!()
    }
    pub fn damage(&self, cost: u32, access: &mut GhostToken<'r>) {
        todo!()
    }
}

fn attack<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
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
pub fn attack2<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
    let hp = &a.hp;
    let ring_ref = &a.rings.borrow(&token)[0];
    
    let damage = a.calculate_damage(d, token);
    let a_energy_cost = a.calculate_attack_cost(d, token);
    let d_energy_cost = d.calculate_defend_cost(a, token);
    a.use_energy(a_energy_cost, token);
    d.use_energy(d_energy_cost, token);
    d.damage(damage , token);
    println!("{:?}", hp.borrow(&token));
    // println!("{:?}", ring_ref);
}

fn attack3<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
    let hp_ref = &d.hp;
    let rings_list_ref = &d.rings;
    let rand_n = a as *const _ as usize;
    let ring_ref = &d.rings.borrow(&token)[rand_n];

    let durability = match &*d.hand.borrow(&token) {
        Hand::Shield { durability } => {
            durability
        }
        Hand::Sword { sharpness } => {
            panic!("irrelevant to the demo :)");
        }
    };
    println!("{:?}", hp_ref.borrow(&token));
    println!("{:?}", rings_list_ref.borrow(&token).len());
    println!("{:?}", ring_ref.borrow(&token).power);
    println!("{:?}", durability.borrow(&token));
}
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
fn attack4<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
    let hp_ref = &d.hp;
    let rings_list_ref = &d.rings;
    let rand_n = a as *const _ as usize;
    let ring_ref = &d.rings.borrow(&token)[rand_n];

    let durability = match &*d.hand.borrow(&token) {
        Hand::Shield { durability } => {
            durability
        }
        Hand::Sword { sharpness } => {
            panic!("irrelevant to the demo :)");
        }
    };
    d.damage(10, token);
    println!("{:?}", hp_ref.borrow(&token));
    println!("{:?}", rings_list_ref.borrow(&token).len());
    // println!("{:?}", ring_ref.borrow(&token).power);
    // println!("{:?}", durability);
}
// fn union_groups<'r>(a: &mut GhostToken<'a>, b: &mut GhostToken<'b>, r: GhostToken<'r>) ->  {
//     GhostToken::new(id)
// }
fn invoke_demo() {
    let entity_a = Entity::new();
    generativity::make_guard!(entity_a_content_group);
    let mut entity_a_content_group = entity_a_content_group;
    generativity::make_guard!(entity_a_group);
    let mut entity_a_group = entity_a_group;
    let entity_a = ghost_cell::GhostCell::new(entity_a);

    let entity_b = Entity::new();
    generativity::make_guard!(entity_b_content_group);
    let mut entity_b_content_group = entity_b_content_group;
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
    make_guard!(a_b_content_union);
    let a_in_group = GhostCell::new(&mut entity_a_content_group);
    let b_in_group = GhostCell::new(&mut entity_b_content_group);
    attack(
        entity_cast_group_mut(&a_in_group, entity_a.borrow_mut(&mut entity_a_group)),
        entity_cast_group_mut(&b_in_group, entity_b.borrow_mut(&mut entity_b_group)),
        &mut a_b_content_union
    );
}

fn complex_example_main() {
    let entities = vec![
        GhostCell::new(Entity::new()),
        GhostCell::new(Entity::new()),
    ];
    generativity::make_guard!(entity_content_group);
    let mut entity_content_group = entity_content_group;
    generativity::make_guard!(entities_content_group);
    let mut entities_content_group = entities_content_group;
    generativity::make_guard!(entities_group);
    let mut entities_group = entities_group;
    let entities = GhostCell::new(entities);
    attack(
        entities.borrow(&entities_group)[0].borrow(&entities_content_group),
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
fn complex_attack<'r>(a: &Entity<'r>, d: &Entity<'r>, token: &mut GhostToken<'r>) {
    let (entity_access, entity_cast) = token_as_entity1_mut(token);
    let open_a = entity_cast(a);
    let armor_ref = match open_a.hand.borrow(&entity_access.hand) {
        Hand::Shield { durability } => {
            durability
        }
        Hand::Sword { sharpness } => {
            panic!("irrelevant to the demo :)");
        }
    };

    complex_power_up_ring(
        open_a, 
        open_a.rings.borrow(&entity_access.rings)[0].borrow_mut(&mut entity_access.rings_content),
        &entity_access.hp,
        &entity_access.rings,
        // &entity_access.rings_content,
        &entity_access.hand,
        &entity_access.hand_content,
        &entity_access.energy,
    );
    *armor_ref.borrow_mut(&mut entity_access.hand_content) += 2;
}
// # Wielder Entity's energy will power up the ring.
// # Changes the ring, but does not change the wielder Entity.
// fn complex_power_up_ring[e: group Entity, mut rr: group Ring = e.rings*](
//     ref[e] entity: Entity,
//     ref[rr] a_ring: Ring
// ):
fn complex_power_up_ring<'l1, 'l2, 'rings_content, 'l3, 'l4, 'l5>(
    entity: &OpenEntity<'l1, 'l2, 'rings_content, 'l3, 'l4, 'l5>,
    a_ring: &'rings_content mut Ring,

    // So rust can't reason about borrows already existing in the sigature
    // - e.rings is lovely - but we can mimic it by just exhaustively listing
    // the disjunction
    token1: &GhostToken<'l1>,
    token2: &GhostToken<'l2>,
    token3: &GhostToken<'l3>,
    token4: &GhostToken<'l4>,
    token5: &GhostToken<'l5>,

) {
    a_ring.power += entity.energy.borrow(token5) / 4
}