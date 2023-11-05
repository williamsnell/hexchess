import random
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import numpy as np
import math

num_samples = 5
num_options = 40

divisors = [1, 4, 20, num_samples, 'even 1', 'even 2']

stats = {
    k: 0 for k in divisors
}

dists = {
    d: [] for d in divisors
}


def random_pick_and_random_fill(num_samples, num_options, divisor, choice_list):
    choices = np.zeros(num_options)

    remaining_samples = num_samples

    n_calls = 0

    while remaining_samples > 0:
        allocated = random.randrange(remaining_samples + 1)
        allocated = (allocated + (divisor - 1)) // divisor

        # remap the available choices based on the distribution parameter

        choices[random.randrange(num_options)] += allocated
        remaining_samples -= allocated
        n_calls += 1
        # choices = np.array([random.random() for i in range(num_options)])

        # choices = (choices * num_samples) / sum(choices)

    stats[divisor] += n_calls
    # print(f"divisor: {divisor} | num_calls: {n_calls}")
    choice_list += [choices]

def divvy_remainder(num_samples, num_options, divisor, choice_list):
    fair_dist_per_option = num_samples // num_options
    remainder = num_samples % num_options
    choices = [fair_dist_per_option for i in range(num_options)]

    # now, divide the remainder randomly
    n_calls = 0
    while remainder > 0:
        n_calls += 1

        allocated = (random.randrange(remainder + 1) + (divisor - 1)) // divisor
        choices[random.randrange(num_options)] += allocated
        remainder -= allocated

    stats[f'even {divisor}'] += n_calls
    choice_list += [choices]

def divvy_remainder(num_samples, num_options, divisor, choice_list):
    fair_dist_per_option = num_samples // num_options
    remainder = num_samples % num_options
    choices = [fair_dist_per_option for i in range(num_options)]

    # now, divide the remainder randomly
    n_calls = 0
    while remainder > 0:
        n_calls += 1

        allocated = (random.randrange(remainder + 1) + (divisor - 1)) // divisor
        choices[random.randrange(num_options)] += allocated
        remainder -= allocated

    stats[f'even {divisor}'] += n_calls
    choice_list += [choices]

# def 
    # generate a random sequence of sorted integers between 0 and num_samples
    # take the difference between neighboring integers
    # return this array of differences

# zip(('cornflowerblue', 'orangered', 'forestgreen', 'olive'),

called = False

def animate(i, axes):
    for (divisor, choice_list), ax in zip(
        dists.items(), axes.flatten()
        ):
        samples_per_frame = 100
        for _ in range(samples_per_frame):
            if isinstance(divisor, str):
                divvy_remainder(num_samples, num_options, int(divisor[5:]), choice_list)
            else:
                random_pick_and_random_fill(num_samples, num_options, divisor, choice_list)
            

        final_dist = np.sum(np.array(choice_list), axis=0)

        # check we're getting the right number of samples out
        assert np.all([sum(a) == num_samples for a in choice_list])

        ax.clear()

        ax.bar(np.arange(num_options), final_dist / samples_per_frame / (i+2), label=divisor, color='g', alpha=1)
        ax.bar(np.arange(num_options), choice_list[-1], label=divisor, color='k', alpha=1)
        ax.set_title(f"range(1, n / {divisor}): \n {stats[divisor] / samples_per_frame / (i+2):.0f} random ints" if not isinstance(divisor, str) else 
                     f"remainder split (1 / {divisor[5:]}), \n {stats[divisor] / samples_per_frame / (i+2):.0f} random ints")

        fig.tight_layout()

    if i > 10:
        return False

fig, axes = plt.subplots(2, 3)
fig.suptitle(f"Distributing {num_samples} samples into {num_options} bins")

ani = FuncAnimation(fig, animate, fargs=[axes], interval=10, frames=20)

ani.save(f"divvy_remainder_{num_samples}.gif")

plt.show()
