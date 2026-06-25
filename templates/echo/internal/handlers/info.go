package handlers

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

func Info(c echo.Context) error {
	return c.JSON(http.StatusOK, map[string]string{
		"service": "api",
		"version": "1.0.0",
	})
}
