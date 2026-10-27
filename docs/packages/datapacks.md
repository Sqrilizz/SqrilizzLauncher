# Datapacks

Datapacks are packages that provide custom gameplay mechanics, recipes, and functions in Minecraft.

## Configuring Datapacks

Datapacks can be installed globally across an instance or placed into specific worlds.

## Smithed Integration

Sqrilizz provides native integration with Smithed, supporting datapacks, dependency resolution, and pack compatibility out of the box.

## Using Specific Worlds

You can configure specific packages to only install a datapack on a specific set of worlds.

+++ App
Select the datapack within the instance packages view and assign target worlds.
+++ CLI
Using the expanded package configuration syntax (check the [configuration reference](../reference/configuring.md) for more info), set the `worlds` field on the package to the list of world names you want to install the datapack in.
+++

## Welding Datapacks

Sqrilizz supports automatic datapack welding to resolve conflicts and merge datapack data tables seamlessly.
