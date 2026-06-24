package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func Info(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"service": "api",
		"version": "1.0.0",
	})
}
