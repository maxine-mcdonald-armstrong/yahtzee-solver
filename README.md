# Yahtzee Solver

Personal project over the Xmas break for learning rust.

## Methodology

I solved this as a DP over a finite-horizon Markov Decision Process. I'm quite fresh to the topic, so this is another learning opportunity.

To improve runtime, we can reduce the state space by considering two transition functions. In a given round, which has a constant scorecard state, we transition between dice states. At the end of a given round, we can choose (or are forced) to transition between scorecard states, with a probability distribution over initial dice-states (our new roll).

In the case of a transition between scorecard states, dp\[s, d\] = max over all valid transitions t(s) (sum over all initial dice states di (dp\[t(s), di\] * P(di)) + score(t(s))).
In the case of a transition between dice states, dp\[s, d\] = max over all k(d) (sum over all valid transitions t(d) given the dice k(d) are kept (dp\[s, t(d)\] * P(t(d)))).

If we iterate from f = 13 to 0, for all scorecard states with f filled values we can solve the dice dp.

### State Space S
So named after "scorecard".

There are a) 64 important values for upper section score, b) for each of the 12 non-yahtzee scores one bit of information: scored or not, and c) for yahtzee three possibilities: scored for 50, yet unscored, or scratched (scored for 0).

```math
\begin{align*}
  S &= 64 * 2^12 * 3 \\
    &= 786432
\end{align*}
```

### State Space D
So named after "dice".

There are a) $$\binom{6+5-1}{5}=252$$ possible dice rolls, b) 3 possible values for 'rolls left' (0, 1, 2).

```math
\begin{align*}
  D &= 252 * 3 \\
    &= 756
\end{align*}
```

## Contributions

I'll consider contributions if they are really cool but if you want to work on this code you're most likely better off making a branch.

## License

Note the `UNLICENSE`.

