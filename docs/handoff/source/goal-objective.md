I want to create a new project called software factory: [https://github.com/djsydney04/software_factory.git](https://github.com/djsydney04/software_factory.git)

Here is the goal:

I want to have a place to build software that feels pretty similair to teh codex app in terms of I can spawn agents and UI is clean. However, I want there to be an integration layer where I can acces s things like cloudflare, my agents have a persistent knowledge base for each softeare factory, and we build a marco harness around codex that allows for persistent reasoning and continued work for the agents for long running tasks, monitoring, and continous self improvment on software projects.&#x20;

The goal is to have some macro goal i.e "Build the greenroom app into the best app possible for music collection and discovery" and then have the agents persistently work on that goal on more microterms, like adding new features, improving UI, fixing bugs, making deployment and ci more efficent etc. Essentially, I want it to work like an optimization problem without adding too much noise.&#x20;

I think I want the job of the human in the factory to be four fold:&#x20;
1\) Monitor progress - see what the agent is working on, what it is thinking, where it is going etc.&#x20;
2\) Steer the agent - perhapes there is a new feature that we want to build, I want to be able to tell the agent htat I am intersted in building that feature. I want it to organize itself so that it can build the best possible product siunce ususally,w I have to continously review things and figure things out but I want the agent ot autonmously do these things.
3\) I want to be able to review the agents work in an incredibly simlpe way, in this case I want there to be app previews, demos, etc. that are attached to PRs (which is how I work through projects and push code) and I want to be able to do so quickly and merge from one exisitng interface.&#x20;
4\) I want hte agents to be persistent. I want to be able to step away from my computer for a day and come back to consistent work from the agents.&#x20;

Use the product design skill and pony tail and the UI lint and I want you to think at a high level with a few other agents about how to buiuld this arcitectually, and then come up with some layouts and then deligate work to sub agents to complete, reivew there findings and come back.&#x20;

I want you to also. do some experimenting and optimize the harness so that it fits with the goals above.&#x20;

In terms of UI, I want two ways to view this:&#x20;
1\)I want an interactive TUI, I want to see agent runs, agent goals, what the main CEO agent is doing and thinking and correct it's reasoning, and amke sure it saves that and conitnues to act on that. I want this to be a palce where you can see at a high level what is going on, and on a micro level also see what is going on.
2\) I want an electron app or whatever the rust equivilent is for an electron app where I can monitor this through a GUI. I want it to ahve the same layout and funcitonality as the TUI, just a differnt way to interface.&#x20;

I want the agents to track there work through a linear like Kanban and comparmentalize the work and use various levels of abstractions to keep track of things.&#x20;

Context is also veyr important. I want to make sure I can connect these agents to as much context as possible when they are building which emans allowing access to CLIS, MCP, and other agent frameworks.&#x20;

I also want agents to work on things with varying degrees of agency and reasoning, lower level takss for smaller features, and high reasoing for larger tasks, but this if for one offs and steering, the main project agent should always be reasoning and building.&#x20;

Also allow me to use agents from mtuiple providers i.e claude, codex, cursor, gemini, and through api, cli, and my actual account for some of possible.&#x20;

I wnat the UI to seem soft and simple. I have included a few examples above of what I would like it to look like. Don't add unesscary text or things.&#x20;

Also I inclkuded the factory AI software factory, this is the general way we should orchestrate the macro harness.&#x20;

I also want to be able to conitue and monitor deployments on like cloudflare and vercel and such:&#x20;

Let me include some articles too for you to gather some insight into what other people in the space are doing&#x20;

[https://builders.ramp.com/post/why-we-built-our-background-agent](https://builders.ramp.com/post/why-we-built-our-background-agent)
[https://www.augmentcode.com/blog/beyond-ai-coding-agents-how-we-built-augments-software-factory](https://www.augmentcode.com/blog/beyond-ai-coding-agents-how-we-built-augments-software-factory)
[https://factory.com/news/software-factory](https://factory.com/news/software-factory)
[https://www.firecrawl.dev/blog/ai-software-factory](https://www.firecrawl.dev/blog/ai-software-factory)
[https://dribbble.com/shots/27086988-AI-Coding-Agent-AI-Code-Editor-AI-Code-Generator-UI-Design](https://dribbble.com/shots/27086988-AI-Coding-Agent-AI-Code-Editor-AI-Code-Generator-UI-Design)

Also, automations would be great.&#x20;

My prefered stack is rust, I think it is great.&#x20;

Referenced image files:
- [Image #1]: images/image-1.png
- [Image #2]: images/image-2.png
- [Image #3]: images/image-3.png
- [Image #4]: images/image-4.png
- [Image #5]: images/image-5.png