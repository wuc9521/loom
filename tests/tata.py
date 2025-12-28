import z3

# -----------------------------
# Utilities (taken from notebook style)
# -----------------------------

def solve_horn(chc, max_unfold=100, verbosity=0):
    z3.set_param(verbose=verbosity)

    s = z3.SolverFor("HORN")
    s.set("engine", "spacer")
    s.set("spacer.max_level", max_unfold)
    s.add(chc)

    res = s.check()
    if res == z3.sat:
        return res, s.model()
    elif res == z3.unsat:
        return res, s.proof()
    else:
        return res, None


# -----------------------------
# Term datatype (a | f(t,t))
# -----------------------------

Term = z3.Datatype("Term")
Term.declare("a")
Term.declare("f", ("l", Term), ("r", Term))
Term = Term.create()

a = Term.a
f = Term.f


# -----------------------------
# Predicates P1..P5
# -----------------------------

P1 = z3.Function("P1", Term, z3.BoolSort())
P2 = z3.Function("P2", Term, z3.BoolSort())
P3 = z3.Function("P3", Term, z3.BoolSort())
P4 = z3.Function("P4", Term, z3.BoolSort())
P5 = z3.Function("P5", Term, z3.BoolSort())


# -----------------------------
# Variables
# -----------------------------

x, y, z = z3.Consts("x y z", Term)


# -----------------------------
# Example 7.6.2 + flattening (7.6.4)
# -----------------------------

chc = []

# (2) P2(a).
chc.append(P2(a))

# (8) P5(f(x,y)) <- P2(x), P2(y)
chc.append(
    z3.ForAll([x, y],
        z3.Implies(
            z3.And(P2(x), P2(y)),
            P5(f(x, y))
        )
    )
)

# (1') P1(f(x,y)) <- P5(x), P2(y)
chc.append(
    z3.ForAll([x, y],
        z3.Implies(
            z3.And(P5(x), P2(y)),
            P1(f(x, y))
        )
    )
)

# (3) P1(f(a,x)) <- P2(x)
chc.append(
    z3.ForAll([x],
        z3.Implies(P2(x), P1(f(a, x)))
    )
)

# (4) P3(f(x,y)) <- P1(x), P2(y)
chc.append(
    z3.ForAll([x, y],
        z3.Implies(
            z3.And(P1(x), P2(y)),
            P3(f(x, y))
        )
    )
)

# (5) P4(x) <- P3(x), P1(x)
chc.append(
    z3.ForAll([x],
        z3.Implies(
            z3.And(P3(x), P1(x)),
            P4(x)
        )
    )
)

# (6) P2(x) <- P4(f(x,y))
chc.append(
    z3.ForAll([x, y],
        z3.Implies(
            P4(f(x, y)),
            P2(x)
        )
    )
)

# (7) P1(y) <- P4(f(x,y))
chc.append(
    z3.ForAll([x, y],
        z3.Implies(
            P4(f(x, y)),
            P1(y)
        )
    )
)


# -----------------------------
# Solve with Spacer
# -----------------------------

res, model = solve_horn(chc)

print("Result:", res)
for P in [P1, P2, P3, P4, P5]:
    print(f"{P.name()}(t) =", model[P])
