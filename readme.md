# Tuigotchi Health

This is a commandline/tui self-care pet application!

Written by Veda Heard and Morgan Hager

# About

Helps with self-care by projecting your needs onto a littol creacher

## Inspiration

When you're focused on something, you often forget to take care of yourself. Even with reminders, sometimes things feel so important that you skip out on vital self-care tasks, keeping you less productive overall. We were inspired by Tamagotchi to create a little creature that eats, drinks, sleeps, and takes care of itself as you do, so that you have that little bit of extra incentive not to skip out on your self-care.

## What it does

It acts as a little creature simulator whose needs line up with yours. The default configuration should work decently well for most people, but it's also possible to configure the times when you do your self-care tasks, and add custom needs to the schedule.

## How we built it

We built it in Rust, using the crossterm library for the terminal escape codes, etc. All the ASCII art was created by hand, and the sound effects were recorded by us.

## Challenges we ran into

Creating the art was very time-consuming, and the timezone math was quite difficult, as it involves conversion from local time to a canonical point in time.

## Accomplishments that we're proud of

- Realistic meows :)
- ASCII animations
- Getting a minimum viable product working on the first day
- Everyone who walked by loved it

## What we learned

- ASCII art is hard (how to make it)
- How to create a TUI from scratch
- Various libraries used in the creation of the project

## What's next for Temp

- Custom sound effects for animals
- Custom animals
- Unlockables
- Pride flag patterns, á la `hyfetch`
