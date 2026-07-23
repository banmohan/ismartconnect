using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace ISmartConnect.Data.Migrations
{
    /// <inheritdoc />
    public partial class initial_schema : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "clients",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    code = table.Column<string>(type: "character varying(100)", maxLength: 100, nullable: false),
                    name = table.Column<string>(type: "character varying(200)", maxLength: 200, nullable: false),
                    api_key = table.Column<string>(type: "character varying(200)", maxLength: 200, nullable: false),
                    is_active = table.Column<bool>(type: "boolean", nullable: false),
                    created_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    updated_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_clients", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "users",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    username = table.Column<string>(type: "character varying(100)", maxLength: 100, nullable: false),
                    password_hash = table.Column<string>(type: "character varying(500)", maxLength: 500, nullable: false),
                    display_name = table.Column<string>(type: "character varying(200)", maxLength: 200, nullable: false),
                    is_active = table.Column<bool>(type: "boolean", nullable: false),
                    created_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    updated_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_users", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "request_logs",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    requested_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    duration_ms = table.Column<int>(type: "integer", nullable: false),
                    http_method = table.Column<string>(type: "character varying(16)", maxLength: 16, nullable: false),
                    path = table.Column<string>(type: "character varying(500)", maxLength: 500, nullable: false),
                    query_string = table.Column<string>(type: "character varying(2000)", maxLength: 2000, nullable: true),
                    client_id = table.Column<Guid>(type: "uuid", nullable: true),
                    client_code = table.Column<string>(type: "character varying(100)", maxLength: 100, nullable: true),
                    ip_address = table.Column<string>(type: "character varying(100)", maxLength: 100, nullable: true),
                    request_headers = table.Column<string>(type: "text", nullable: true),
                    request_body = table.Column<string>(type: "text", nullable: true),
                    status_code = table.Column<int>(type: "integer", nullable: false),
                    response_body = table.Column<string>(type: "text", nullable: true),
                    is_error = table.Column<bool>(type: "boolean", nullable: false),
                    error_message = table.Column<string>(type: "character varying(2000)", maxLength: 2000, nullable: true),
                    exception_details = table.Column<string>(type: "text", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_request_logs", x => x.id);
                    table.ForeignKey(
                        name: "request_logs_client_id_fkey",
                        column: x => x.client_id,
                        principalTable: "clients",
                        principalColumn: "id",
                        onDelete: ReferentialAction.SetNull);
                });

            migrationBuilder.CreateIndex(
                name: "clients_api_key_uidx",
                table: "clients",
                column: "api_key",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "clients_code_uidx",
                table: "clients",
                column: "code",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_request_logs_client_id",
                table: "request_logs",
                column: "client_id");

            migrationBuilder.CreateIndex(
                name: "request_logs_client_code_idx",
                table: "request_logs",
                column: "client_code");

            migrationBuilder.CreateIndex(
                name: "request_logs_is_error_idx",
                table: "request_logs",
                column: "is_error");

            migrationBuilder.CreateIndex(
                name: "request_logs_path_idx",
                table: "request_logs",
                column: "path");

            migrationBuilder.CreateIndex(
                name: "request_logs_requested_at_idx",
                table: "request_logs",
                column: "requested_at",
                descending: new[] { true });

            migrationBuilder.CreateIndex(
                name: "request_logs_status_code_idx",
                table: "request_logs",
                column: "status_code");

            migrationBuilder.CreateIndex(
                name: "users_username_uidx",
                table: "users",
                column: "username",
                unique: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "request_logs");

            migrationBuilder.DropTable(
                name: "users");

            migrationBuilder.DropTable(
                name: "clients");
        }
    }
}
