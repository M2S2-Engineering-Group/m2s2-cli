package handlers

import "github.com/gofiber/fiber/v2"

func Info(c *fiber.Ctx) error {
	return c.JSON(fiber.Map{
		"service": "api",
		"version": "1.0.0",
	})
}
